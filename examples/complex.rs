use std::{
    ffi::{CString, c_int},
    os::fd::{BorrowedFd, IntoRawFd},
    sync::{Arc, RwLock},
};

use anyhow::Result;
use colpetto::{Libinput, event::AsRawEvent};
use crossbeam::channel::Receiver;
use saddle::Seat;
use tokio::{
    pin,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::LocalSet,
};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let seat = Seat::new().await?;

    let has_control = Arc::new(RwLock::new(true));

    info!("Spawning channels");

    let (ask_sx, respond_rx) = {
        let seat = seat.clone();

        let (ask_sx, ask_rx) = mpsc::unbounded_channel::<CString>();
        let (respond_sx, respond_rx) = crossbeam::channel::unbounded::<c_int>();

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

        let (close_sx, close_rx) = mpsc::unbounded_channel::<c_int>();

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
            EventType::Keyboard => {
                // Check if we have control
                if *has_control.read().unwrap() {
                    info!("Keyboard event received, switching to VT 2");

                    if let Err(e) = seat.switch_session(2).await {
                        error!("Failed to switch to VT 2: {}", e);
                    }
                } else {
                    debug!("Keyboard event received but we don't have control");
                }
            }
            _ => {}
        }
    }

    libinput_signal_handle.send(LibinputSignal::Shutdown)?;

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
    Keyboard,
    Unknown,
}

impl From<&colpetto::Event> for EventType {
    fn from(value: &colpetto::Event) -> Self {
        match value {
            colpetto::Event::Keyboard(_) => EventType::Keyboard,
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
    ask_sx: UnboundedSender<CString>,
    close_sx: UnboundedSender<i32>,
    respond_rx: Receiver<i32>,
) -> Result<(
    UnboundedReceiver<Result<Event, colpetto::Error>>,
    UnboundedSender<LibinputSignal>,
)> {
    let (event_sx, event_rx) = mpsc::unbounded_channel();
    let (signal_sx, mut signal_rx) = mpsc::unbounded_channel();

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
