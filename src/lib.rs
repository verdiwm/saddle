use std::{
    ffi::CString,
    io,
    os::fd::{BorrowedFd, OwnedFd},
};

use futures_core::Stream;
use login1::{manager::ManagerProxy, seat::SeatProxy, session::SessionProxy};
use rustix::fs;
use tokio_stream::StreamExt;
use tracing::debug;
use zbus::Connection;

pub mod login1;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Zbus(#[from] zbus::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl From<rustix::io::Errno> for Error {
    fn from(value: rustix::io::Errno) -> Self {
        Self::Io(value.into())
    }
}

#[derive(Clone)]
pub struct Seat {
    session: SessionProxy<'static>,
    seat: SeatProxy<'static>,
    seat_name: String,
}

impl Seat {
    pub async fn new() -> Result<Self> {
        debug!("Connecting to system bus");
        let connection = Connection::system().await?;

        debug!("Creating manager proxy");
        let manager = ManagerProxy::new(&connection).await?;

        let session = manager.get_session_by_pid(std::process::id()).await?;

        debug!("Creating session proxy");
        let session = SessionProxy::new(&connection, session).await?;

        debug!("Trying to take control of the session");
        session.take_control(false).await?;

        let (seat_name, seat_object) = session.seat().await?;

        let seat = SeatProxy::new(&connection, seat_object).await?;

        Ok(Self {
            session,
            seat,
            seat_name,
        })
    }

    pub async fn active_stream(&self) -> impl Stream<Item = Result<bool>> {
        let active_changed = self.session.receive_active_changed().await;

        active_changed.then(|prop| async move { prop.get().await.map_err(Error::Zbus) })
    }

    pub fn seat_name(&self) -> &str {
        self.seat_name.as_str()
    }

    pub async fn release_session(&self) -> Result<()> {
        self.session.release_control().await?;

        Ok(())
    }

    pub async fn aquire_session(&self) -> Result<()> {
        debug!("Trying to take control of the session");
        self.session.take_control(false).await?;

        Ok(())
    }

    pub async fn switch_session(&self, session: u32) -> Result<()> {
        self.seat.switch_to(session).await?;

        Ok(())
    }

    pub async fn open_device<D: AsDevice>(&self, device: D) -> Result<OwnedFd> {
        let (major, minor) = device.as_device()?;

        let (fd, _) = self.session.take_device(major, minor).await?;

        Ok(fd.into())
    }

    pub async fn close_device<D: AsDevice>(&self, device: D) -> Result<()> {
        let (major, minor) = device.as_device()?;

        self.session.release_device(major, minor).await?;

        Ok(())
    }
}

pub unsafe trait AsDevice {
    fn as_device(&self) -> Result<(u32, u32)>;
}

macro_rules! impl_as_device_arg {
    ($($ty:ident)+) => {
       $(
            unsafe impl AsDevice for $ty {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::stat(self)?;

                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }
       )*
    };
    ($($ty:ident < $lt:lifetime >)+) => {
        $(
             unsafe impl AsDevice for $ty<$lt> {
                 fn as_device(&self) -> Result<(u32, u32)> {
                     let stat = fs::stat(self)?;

                     Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                 }
             }
        )*
     };
}

macro_rules! impl_as_device_fd {
    ($($ty:ident)+) => {
        $(
            unsafe impl AsDevice for $ty {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::fstat(self)?;

                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }
        )*
    };
    ($($ty:ident < $lt:lifetime >)+) => {
        $(
            unsafe impl AsDevice for $ty<$lt> {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::fstat(self)?;

                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }
        )*
    };
}

impl_as_device_arg!(CString);
impl_as_device_fd!(BorrowedFd<'_>);
