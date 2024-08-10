use std::{
    ffi::{c_int, CString},
    time::Duration,
};

use anyhow::Result as AnyResult;
use colpetto::{sys, Libinput};
use futures_util::{StreamExt, TryStreamExt};
use reconciler::handler::EventHandler;
use rustix::{
    fd::{BorrowedFd, IntoRawFd, OwnedFd},
    fs,
    process::getpid,
};
use saddle::login1::{manager::ManagerProxy, seat::SeatProxy, session::SessionProxy};
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, error};
use zbus::Connection;

#[tokio::main]
async fn main() -> AnyResult<()> {
    tracing_subscriber::fmt::init();

    let connection = Connection::system().await?;

    let manager = ManagerProxy::new(&connection).await?;

    let session = manager
        .get_session_by_pid(getpid().as_raw_nonzero().get() as u32)
        .await?;

    let session = SessionProxy::new(&connection, session).await?;

    let (seat_name, seat_object) = session.seat().await?;

    dbg!(&seat_name);

    let seat = SeatProxy::new(&connection, seat_object).await?;

    session
        .take_control(false)
        .await
        .expect("Failed to take control of session");

    let controlling = true;

    println!("Creating channels");

    let (ask_sx, ask_rx) = mpsc::unbounded_channel::<CString>();
    let (respond_sx, respond_rx) = crossbeam::channel::unbounded::<c_int>();

    let mut ask_rx = UnboundedReceiverStream::new(ask_rx);

    tokio::spawn({
        let session = session.clone();
        async move {
            while let Some(path) = ask_rx.next().await {
                let stat = fs::stat(path).unwrap();

                let major = fs::major(stat.st_rdev);
                let minor = fs::minor(stat.st_rdev);

                match session.take_device(major, minor).await {
                    Ok((device, _)) => {
                        let fd: OwnedFd = device.into();
                        respond_sx.send(fd.into_raw_fd()).unwrap();
                    }
                    Err(err) => {
                        error!("Failed to open device");
                        dbg!(err);
                        respond_sx.send(-1).unwrap()
                    }
                }
            }
        }
    });

    let (close_sx, close_rx) = mpsc::unbounded_channel::<c_int>();

    let mut close_rx = UnboundedReceiverStream::new(close_rx);

    tokio::spawn({
        let session = session.clone();
        async move {
            while let Some(fd) = close_rx.next().await {
                let stat = fs::fstat(unsafe { BorrowedFd::borrow_raw(fd) }).unwrap();

                let major = fs::major(stat.st_rdev);
                let minor = fs::minor(stat.st_rdev);

                let _ = session.release_device(major, minor);
            }
        }
    });

    println!("Creating libinput");

    let libinput = Libinput::new(
        move |path, _| {
            debug!("Opening fd at path {}", path.to_string_lossy());
            ask_sx.send(path.to_owned()).unwrap();
            let res = respond_rx.recv().unwrap();

            res
        },
        move |fd| {
            debug!("Closing fd: {fd}");
            let _ = close_sx.send(fd);
        },
    )?;

    println!("Assigning seat");

    libinput.assign_seat(CString::new(seat_name).unwrap().as_c_str())?;

    println!("Starting loop");

    let mut active_recv = session.receive_active_changed().await;

    tokio::spawn({
        let libinput = libinput.clone();

        async move {
            while let Some(active) = active_recv.next().await {
                debug!("active changed");

                if let Ok(active) = active.get().await {
                    if active {
                        debug!("Resuming libinput");
                        unsafe { sys::libinput_resume(libinput.as_raw()) };
                    } else {
                        debug!("Suspending libinput");
                        unsafe { sys::libinput_suspend(libinput.as_raw()) };
                    }
                }
            }
        }
    });

    let mut event_stream = libinput.event_stream()?;

    // FAILSAFE
    tokio::spawn({
        let session = session.clone();

        async move {
            sleep(Duration::from_secs(20)).await;

            let _ = session.release_control().await;
        }
    });

    let _event_handler = EventHandler::new();

    while let Some(event) = event_stream.try_next().await? {
        dbg!(&event);
        match event {
            colpetto::Event::Keyboard(_) => {
                if controlling {
                    println!("Switching");
                    seat.switch_to(1).await?;
                }
            }
            _ => {}
        }
    }

    println!("Finished loop");

    Ok(())
}
