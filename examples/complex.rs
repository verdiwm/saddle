use std::{
    ffi::{CString, c_int},
    os::fd::{BorrowedFd, IntoRawFd},
    sync::mpsc,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use colpetto::{
    Libinput,
    event::{AsRawEvent, KeyboardEvent},
};
use input_linux_sys::{KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_ESC};
use saddle::Seat;
use tokio::{
    pin,
    sync::mpsc::{self as tokio_mpsc},
    task::LocalSet,
};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use tracing::{debug, error, info};

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

    tokio::spawn({
        let seat = seat.clone();
        let has_control = has_control.clone();
        let libinput_signal_handle = libinput_signal_handle.clone();

        async move {
            let stream = seat.active_stream().await;

            pin!(stream);

            while let Some(is_active) = stream.try_next().await? {
                if is_active {
                    info!("Session became active, taking control");
                    seat.aquire_session().await?;
                    *has_control.write().unwrap() = true;
                    libinput_signal_handle.send(LibinputSignal::Resume)?;
                } else {
                    info!("Session became inactive");
                    seat.release_session().await?;
                    *has_control.write().unwrap() = false;
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
            EventType::Keyboard(key) => {
                // Check if we have control
                if *has_control.read().unwrap() {
                    match key as c_int {
                        KEY_1 => switch(&seat, 1).await?,
                        KEY_2 => switch(&seat, 2).await?,
                        KEY_3 => switch(&seat, 3).await?,
                        KEY_4 => switch(&seat, 4).await?,
                        KEY_5 => switch(&seat, 5).await?,
                        KEY_6 => switch(&seat, 6).await?,
                        KEY_7 => switch(&seat, 7).await?,
                        KEY_8 => switch(&seat, 8).await?,
                        KEY_9 => switch(&seat, 9).await?,
                        KEY_ESC => {
                            info!("Exiting...");
                            libinput_signal_handle.send(LibinputSignal::Shutdown)?;
                        }
                        _ => {}
                    }
                } else {
                    debug!("Keyboard event received but we don't have control");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn switch(seat: &Seat, session: u32) -> Result<()> {
    info!("Keyboard event received, switching to VT {session}");

    if let Err(e) = seat.switch_session(session).await {
        error!("Failed to switch to VT 2: {}", e);
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
    Keyboard(u32),
    Unknown,
}

impl From<&colpetto::Event> for EventType {
    fn from(value: &colpetto::Event) -> Self {
        match value {
            colpetto::Event::Keyboard(KeyboardEvent::Key(event)) => {
                EventType::Keyboard(event.key())
            }
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
