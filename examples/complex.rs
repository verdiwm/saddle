use std::{
    collections::{HashMap, HashSet},
    ffi::CString,
    os::fd::{BorrowedFd, IntoRawFd},
    sync::Arc,
};

use anyhow::Result;
use colpetto::{
    event::KeyState,
    helper::{EventType, Handle as LibinputHandle},
};
use input_linux_sys::{
    KEY_ESC, KEY_F1, KEY_F2, KEY_F3, KEY_F4, KEY_F5, KEY_F6, KEY_F7, KEY_F8, KEY_F9, KEY_LEFTALT,
    KEY_LEFTCTRL, KEY_RIGHTALT, KEY_RIGHTCTRL,
};
use saddle::Seat;
use tokio::{pin, sync::RwLock};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, trace};

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
    let seat_name = CString::new(seat.seat_name()).expect("Invalid seat name");

    let has_control = Arc::new(RwLock::new(false));

    let (libinput_handle, mut event_stream) = {
        let open_seat = seat.clone();
        let close_seat = seat.clone();

        LibinputHandle::new(
            move |path| {
                let seat = open_seat.clone();

                async move {
                    match seat.open_device(path).await {
                        Ok(fd) => fd.into_raw_fd(),
                        Err(err) => {
                            error!("Failed to open device: {err}");
                            -1
                        }
                    }
                }
            },
            move |fd| {
                let seat = close_seat.clone();

                async move {
                    let _ = seat.close_device(unsafe { BorrowedFd::borrow_raw(fd) });
                }
            },
            seat_name,
        )?
    };

    let key_map = KeyMap::new();
    let modifier_state = Arc::new(RwLock::new(ModifierState::new()));

    tokio::spawn({
        let seat = seat.clone();
        let has_control = has_control.clone();
        let libinput_handle = libinput_handle.clone();
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
                    libinput_handle.resume()?;
                } else {
                    info!("Session became inactive");
                    seat.release_session().await?;
                    *has_control.write().await = false;
                    libinput_handle.suspend()?;
                }
            }

            anyhow::Ok(())
        }
    });

    while let Some(event) = event_stream.try_next().await? {
        trace!(
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
                        libinput_handle.shutdown()?;
                    }

                    // Only process function keys when Ctrl+Alt are held
                    if modifier_state.read().await.is_ctrl_alt_pressed() {
                        if let Some(vt) = key_map.get_vt(key) {
                            if *has_control.read().await {
                                info!("Ctrl+Alt+F{vt} pressed, switching to VT {vt}");

                                if let Err(e) = seat.switch_session(vt).await {
                                    error!("Failed to switch to VT {vt}: {e}");
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
