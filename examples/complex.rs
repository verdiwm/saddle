use std::{
    collections::{HashMap, HashSet},
    ffi::{CString, c_int},
    os::fd::{BorrowedFd, IntoRawFd},
    sync::{Arc, mpsc},
};

use anyhow::Result;
use colpetto::{
    Libinput,
    event::{AsRawEvent, KeyState, KeyboardEvent},
};
use input_linux_sys::{
    KEY_ESC, KEY_F1, KEY_F2, KEY_F3, KEY_F4, KEY_F5, KEY_F6, KEY_F7, KEY_F8, KEY_F9, KEY_LEFTALT,
    KEY_LEFTCTRL, KEY_RIGHTALT, KEY_RIGHTCTRL,
};
use saddle::Seat;
use tokio::{
    pin,
    sync::{
        RwLock,
        mpsc::{self as tokio_mpsc},
    },
    task::LocalSet,
};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::{debug, error, info};

/// Maps function keys to VT numbers
struct KeyMap {
    mappings: HashMap<u32, u32>,
}

impl KeyMap {
    fn new() -> Self {
        let mut mappings = HashMap::new();

        // Function keys mapped to respective VTs
        mappings.insert(KEY_F1 as u32, 1);
        mappings.insert(KEY_F2 as u32, 2);
        mappings.insert(KEY_F3 as u32, 3);
        mappings.insert(KEY_F4 as u32, 4);
        mappings.insert(KEY_F5 as u32, 5);
        mappings.insert(KEY_F6 as u32, 6);
        mappings.insert(KEY_F7 as u32, 7);
        mappings.insert(KEY_F8 as u32, 8);
        mappings.insert(KEY_F9 as u32, 9);

        Self { mappings }
    }

    fn get_vt(&self, key: u32) -> Option<u32> {
        self.mappings.get(&key).copied()
    }
}

struct ModifierState {
    pressed_keys: HashSet<u32>,
}

impl ModifierState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    fn update(&mut self, key: u32, state: KeyState) {
        match state {
            KeyState::Pressed => {
                self.pressed_keys.insert(key);
            }
            KeyState::Released => {
                self.pressed_keys.remove(&key);
            }
        }
    }

    fn is_ctrl_pressed(&self) -> bool {
        self.pressed_keys.contains(&(KEY_LEFTCTRL as u32))
            || self.pressed_keys.contains(&(KEY_RIGHTCTRL as u32))
    }

    fn is_alt_pressed(&self) -> bool {
        self.pressed_keys.contains(&(KEY_LEFTALT as u32))
            || self.pressed_keys.contains(&(KEY_RIGHTALT as u32))
    }

    fn is_ctrl_alt_pressed(&self) -> bool {
        self.is_ctrl_pressed() && self.is_alt_pressed()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let seat = Seat::new().await?;

    let has_control = Arc::new(RwLock::new(false));

    info!("Spawning channels");

    let (ask_sx, respond_rx) = {
        let seat = seat.clone();

        let (ask_sx, ask_rx) = tokio_mpsc::unbounded_channel::<CString>();
        let (respond_sx, respond_rx) = mpsc::channel::<c_int>();

        let mut ask_rx = UnboundedReceiverStream::new(ask_rx);

        tokio::spawn(async move {
            while let Some(path) = ask_rx.next().await {
                match seat.open_device(path).await {
                    Ok(fd) => {
                        respond_sx.send(fd.into_raw_fd())?;
                    }
                    Err(err) => {
                        error!("Failed to open device: {err}");
                        respond_sx.send(-1)?
                    }
                }
            }

            anyhow::Ok(())
        });

        (ask_sx, respond_rx)
    };

    let close_sx = {
        let seat = seat.clone();

        let (close_sx, close_rx) = tokio_mpsc::unbounded_channel::<c_int>();

        let mut close_rx = UnboundedReceiverStream::new(close_rx);

        tokio::spawn(async move {
            while let Some(fd) = close_rx.next().await {
                let _ = seat
                    .close_device(unsafe { BorrowedFd::borrow_raw(fd) })
                    .await;
            }

            anyhow::Ok(())
        });

        close_sx
    };

    let (rx, libinput_signal_handle) = spawn_libinput_task(
        CString::new(seat.seat_name()).expect("Invalid seat name"),
        ask_sx,
        close_sx,
        respond_rx,
    )?;

    let key_map = KeyMap::new();
    let modifier_state = Arc::new(RwLock::new(ModifierState::new()));

    tokio::spawn({
        let seat = seat.clone();
        let has_control = has_control.clone();
        let libinput_signal_handle = libinput_signal_handle.clone();
        let modifier_state = modifier_state.clone();

        async move {
            let stream = seat.active_stream().await;

            pin!(stream);

            while let Some(is_active) = stream.try_next().await? {
                if is_active {
                    info!("Session became active, taking control");
                    seat.aquire_session().await?;
                    *has_control.write().await = true;

                    // Reset modifier state when session becomes active to avoid stuck keys
                    *modifier_state.write().await = ModifierState::new();

                    libinput_signal_handle.send(LibinputSignal::Resume)?;
                } else {
                    info!("Session became inactive");
                    seat.release_session().await?;
                    *has_control.write().await = false;
                    libinput_signal_handle.send(LibinputSignal::Suspend)?;
                }
            }

            anyhow::Ok(())
        }
    });

    let mut stream = UnboundedReceiverStream::new(rx);

    while let Some(event) = stream.try_next().await? {
        println!(
            "Got \"{}\" event from \"{}\"",
            event.name, event.device_name
        );

        match event.event_type {
            EventType::Keyboard { key, state } => {
                modifier_state.write().await.update(key, state);

                if state == KeyState::Pressed {
                    // Handle ESC for exit
                    if key as i32 == KEY_ESC {
                        info!("ESC pressed, exiting...");
                        libinput_signal_handle.send(LibinputSignal::Shutdown)?;
                    }

                    // Only process function keys when Ctrl+Alt are held
                    if modifier_state.read().await.is_ctrl_alt_pressed() {
                        if let Some(vt) = key_map.get_vt(key) {
                            if *has_control.read().await {
                                info!("Ctrl+Alt+F{} pressed, switching to VT {}", vt, vt);

                                if let Err(e) = seat.switch_session(vt).await {
                                    error!("Failed to switch to VT 2: {}", e);
                                }
                            } else {
                                debug!("Not switching VT - session inactive");
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Event {
    name: &'static str,
    event_type: EventType,
    device_name: String,
}

#[derive(Debug)]
enum EventType {
    Keyboard { key: u32, state: KeyState },
    Unknown,
}

impl From<&colpetto::Event> for EventType {
    fn from(value: &colpetto::Event) -> Self {
        match value {
            colpetto::Event::Keyboard(KeyboardEvent::Key(event)) => EventType::Keyboard {
                key: event.key(),
                state: event.key_state(),
            },
            _ => EventType::Unknown,
        }
    }
}

enum LibinputSignal {
    Shutdown,
    Suspend,
    Resume,
}

fn spawn_libinput_task(
    seat_name: CString,
    ask_sx: tokio_mpsc::UnboundedSender<CString>,
    close_sx: tokio_mpsc::UnboundedSender<i32>,
    respond_rx: mpsc::Receiver<i32>,
) -> Result<(
    tokio_mpsc::UnboundedReceiver<Result<Event, colpetto::Error>>,
    tokio_mpsc::UnboundedSender<LibinputSignal>,
)> {
    let (event_sx, event_rx) = tokio_mpsc::unbounded_channel();
    let (signal_sx, mut signal_rx) = tokio_mpsc::unbounded_channel();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    std::thread::spawn(move || {
        let local = LocalSet::new();

        local.spawn_local(async move {
            info!("Creating libinput object");

            let mut libinput = Libinput::new(
                move |path, _| {
                    debug!("Opening fd at path {}", path.to_string_lossy());
                    ask_sx.send(path.to_owned()).unwrap();
                    let res = respond_rx.recv().unwrap();

                    Ok(res)
                },
                move |fd| {
                    debug!("Closing fd: {fd}");
                    let _ = close_sx.send(fd); // Libinput doesn't care about closing errors
                },
            )?;

            libinput.udev_assign_seat(&seat_name)?;

            let mut stream = libinput.event_stream()?;

            loop {
                tokio::select! {
                    Some(signal) = signal_rx.recv() => {
                        match signal {
                            LibinputSignal::Shutdown => break,
                            LibinputSignal::Suspend => libinput.suspend(),
                            LibinputSignal::Resume => libinput.resume().unwrap(), // FIXME: error handling
                        }
                    }
                    Some(res) = stream.next() => {
                        if event_sx
                            .send(res.map(|ref event| Event {
                                name: event.event_type(),
                                event_type: event.into(),
                                device_name: event.device().name().to_string_lossy().to_string(),
                            }))
                            .is_err()
                        {
                            break;
                        }
                    }
                    else => break,
                }
            }

            anyhow::Ok(())
        });

        rt.block_on(local);
    });

    Ok((event_rx, signal_sx))
}
