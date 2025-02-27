/// Saddle: An asynchronous library for Linux user seat management.
///
/// Saddle provides a simple, future-based API for managing user sessions and
/// securely accessing devices without requiring root privileges. It's designed
/// as a Rust alternative to libseat with strong async support.
///
/// # Key features
///
/// - Session management (obtain, release, and monitor sessions)
/// - Secure device access for non-privileged applications
/// - Virtual terminal (VT) switching
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use saddle::Seat;
///
/// #[tokio::main]
/// async fn main() -> saddle::Result<()> {
///     // Connect to the current session
///     let seat = Seat::new().await?;
///     println!("Connected to seat: {}", seat.seat_name());
///
///     // Open a device
///     let fd = seat.open_device("/dev/input/event0").await?;
///
///     // Monitor session state changes
///     let mut stream = seat.active_stream().await;
///     while let Some(is_active) = stream.try_next().await? {
///         if is_active {
///             println!("Session became active");
///             seat.aquire_session().await?;
///         } else {
///             println!("Session became inactive");
///             seat.release_session().await?;
///         }
///     }
///
///     // Close device and release session when done
///     seat.close_device("/dev/input/event0").await?;
///     seat.release_session().await?;
///
///     Ok(())
/// }
/// ```
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    fs::File,
    io,
    os::fd::{BorrowedFd, OwnedFd},
    path::{Path, PathBuf},
};

use login1::{manager::ManagerProxy, seat::SeatProxy, session::SessionProxy};
use rustix::fs;
use tokio_stream::{Stream, StreamExt};
use tracing::debug;
use zbus::Connection;

pub mod login1;

/// Library result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can occur during seat operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// D-Bus communication errors (used by the logind backend).
    #[error("{0}")]
    Zbus(#[from] zbus::Error),

    /// I/O errors during device operations.
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl From<rustix::io::Errno> for Error {
    fn from(value: rustix::io::Errno) -> Self {
        Self::Io(value.into())
    }
}

/// Represents a user seat with session control capabilities.
#[derive(Clone)]
pub struct Seat {
    session: SessionProxy<'static>,
    seat: SeatProxy<'static>,
    seat_name: String,
}

impl Seat {
    /// Creates a new seat connection for the current process.
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

    /// Returns a stream that emits when session active state changes.
    /// Applications should monitor this stream to know when to pause or resume device processing.
    ///
    /// When the session becomes inactive, applications should release the session
    /// using `release_session()`. When it becomes active again, they should
    /// re-acquire it using `aquire_session()`.
    pub async fn active_stream(&self) -> impl Stream<Item = Result<bool>> {
        tokio_stream::once(Ok(true)).chain(
            self.session
                .receive_active_changed()
                .await
                .then(|prop| async move { prop.get().await.map_err(Error::Zbus) }),
        )
    }

    /// Returns the seat name (e.g., "seat0").
    ///
    /// This can be useful for applications that need to identify the seat
    /// they're running on, such as when integrating with libinput.
    pub fn seat_name(&self) -> &str {
        self.seat_name.as_str()
    }

    /// Releases control of the session.
    ///
    /// Applications should call this when:
    /// - The session becomes inactive (e.g., when switching VTs)
    /// - They no longer need access to the session's devices
    ///
    /// After releasing control, calls to `open_device()` will fail until
    /// control is re-acquired with `aquire_session()`.
    pub async fn release_session(&self) -> Result<()> {
        self.session.release_control().await?;

        Ok(())
    }

    /// Takes control of the session.
    ///
    /// Re-acquires control of a previously released session. This should be called:
    /// - When the session becomes active again after being inactive
    /// - Before trying to open devices after releasing the session
    pub async fn aquire_session(&self) -> Result<()> {
        debug!("Trying to take control of the session");
        self.session.take_control(false).await?;

        Ok(())
    }

    /// Switches to a different VT session.
    ///
    /// The provided session number corresponds to the VT number
    /// (e.g., 1 for tty1, 7 for tty7). This is commonly used to implement
    /// Alt+Ctrl+Fn key combinations for VT switching.
    ///
    /// Note that this operation requires the current session to be active.
    /// If the session is inactive, this will return an error.
    pub async fn switch_session(&self, session: u32) -> Result<()> {
        self.seat.switch_to(session).await?;

        Ok(())
    }

    /// Opens a device securely, returning a file descriptor.
    ///
    /// This allows non-privileged applications to access devices that
    /// would normally require elevated permissions. The method works by
    /// requesting access from the session manager, which grants temporary
    /// access to the specified device.
    ///
    /// The returned file descriptor can be used directly or passed to
    /// libraries like libinput.
    ///
    /// # Examples
    ///
    /// Opening a specific input device:
    ///
    /// ```
    /// let fd = seat.open_device("/dev/input/event0").await?;
    /// // Use the file descriptor with other libraries
    /// ```
    ///
    /// Opening a device from an existing file descriptor:
    ///
    /// ```
    /// let fd = seat.open_device(existing_fd).await?;
    /// ```
    pub async fn open_device<D: AsDevice>(&self, device: D) -> Result<OwnedFd> {
        let (major, minor) = device.as_device()?;

        let (fd, _) = self.session.take_device(major, minor).await?;

        Ok(fd.into())
    }

    /// Closes a previously opened device.
    ///
    /// Applications should close devices when they no longer need them
    /// to release system resources.
    pub async fn close_device<D: AsDevice>(&self, device: D) -> Result<()> {
        let (major, minor) = device.as_device()?;

        self.session.release_device(major, minor).await?;

        Ok(())
    }
}

/// Trait for types that can identify a device.
///
/// This trait is implemented for various path-like types (e.g., strings, paths)
/// and file descriptor types, allowing them to be used with `open_device` and
/// `close_device` methods.
///
/// # Safety
///
/// This trait is marked as unsafe because implementations must guarantee that
/// they return valid device numbers.
pub unsafe trait AsDevice {
    /// Returns major and minor device numbers for the device.
    fn as_device(&self) -> Result<(u32, u32)>;
}

/// Implement AsDevice for a type and a reference to that type using stat()
macro_rules! impl_as_device_stat {
    ($($ty:ty),* $(,)?) => {
        $(
            // Implement for the type itself
            unsafe impl AsDevice for $ty {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::stat(self)?;
                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }

            // Implement for a reference to the type
            unsafe impl AsDevice for &$ty {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::stat(*self)?;
                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }
        )*
    };
}

/// Implement AsDevice for a type and a reference to that type using fstat()
macro_rules! impl_as_device_fstat {
    ($($ty:ty),* $(,)?) => {
        $(
            // Implement for the type itself
            unsafe impl AsDevice for $ty {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::fstat(self)?;
                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }

            // Implement for a reference to the type
            unsafe impl AsDevice for &$ty {
                fn as_device(&self) -> Result<(u32, u32)> {
                    let stat = fs::fstat(*self)?;
                    Ok((fs::major(stat.st_rdev), fs::minor(stat.st_rdev)))
                }
            }
        )*
    };
}

// Implement for path-like types
impl_as_device_stat! {
    CString,
    CStr,
    PathBuf,
    Path,
    String,
    str,
    OsString,
    OsStr
}

// Implement for file descriptor types
impl_as_device_fstat! {
    OwnedFd,
    BorrowedFd<'_>,
    File
}
