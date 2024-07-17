//! # D-Bus interface proxy for: `org.freedesktop.login1.Session`
use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1"
)]
trait Session {
    /// Activate method
    fn activate(&self) -> zbus::Result<()>;

    /// Kill method
    fn kill(&self, who: &str, signal_number: i32) -> zbus::Result<()>;

    /// Lock method
    fn lock(&self) -> zbus::Result<()>;

    /// PauseDeviceComplete method
    fn pause_device_complete(&self, major: u32, minor: u32) -> zbus::Result<()>;

    /// ReleaseControl method
    fn release_control(&self) -> zbus::Result<()>;

    /// ReleaseDevice method
    fn release_device(&self, major: u32, minor: u32) -> zbus::Result<()>;

    /// SetBrightness method
    fn set_brightness(&self, subsystem: &str, name: &str, brightness: u32) -> zbus::Result<()>;

    /// SetClass method
    fn set_class(&self, class: &str) -> zbus::Result<()>;

    /// SetDisplay method
    fn set_display(&self, display: &str) -> zbus::Result<()>;

    /// SetIdleHint method
    fn set_idle_hint(&self, idle: bool) -> zbus::Result<()>;

    /// SetLockedHint method
    fn set_locked_hint(&self, locked: bool) -> zbus::Result<()>;

    /// SetTTY method
    #[zbus(name = "SetTTY")]
    fn set_tty(&self, tty_fd: zbus::zvariant::Fd<'_>) -> zbus::Result<()>;

    /// SetType method
    fn set_type(&self, type_: &str) -> zbus::Result<()>;

    /// TakeControl method
    fn take_control(&self, force: bool) -> zbus::Result<()>;

    /// TakeDevice method
    fn take_device(&self, major: u32, minor: u32) -> zbus::Result<(zbus::zvariant::OwnedFd, bool)>;

    /// Terminate method
    fn terminate(&self) -> zbus::Result<()>;

    /// Unlock method
    fn unlock(&self) -> zbus::Result<()>;

    /// Lock signal
    #[zbus(signal)]
    fn lock(&self) -> zbus::Result<()>;

    /// PauseDevice signal
    #[zbus(signal)]
    fn pause_device(&self, major: u32, minor: u32, type_: &str) -> zbus::Result<()>;

    /// ResumeDevice signal
    #[zbus(signal)]
    fn resume_device(&self, major: u32, minor: u32, fd: zbus::zvariant::Fd<'_>)
        -> zbus::Result<()>;

    /// Unlock signal
    #[zbus(signal)]
    fn unlock(&self) -> zbus::Result<()>;

    /// Active property
    #[zbus(property)]
    fn active(&self) -> zbus::Result<bool>;

    /// Audit property
    #[zbus(property)]
    fn audit(&self) -> zbus::Result<u32>;

    /// Class property
    #[zbus(property)]
    fn class(&self) -> zbus::Result<String>;

    /// Desktop property
    #[zbus(property)]
    fn desktop(&self) -> zbus::Result<String>;

    /// Display property
    #[zbus(property)]
    fn display(&self) -> zbus::Result<String>;

    /// Id property
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    /// IdleHint property
    #[zbus(property)]
    fn idle_hint(&self) -> zbus::Result<bool>;

    /// IdleSinceHint property
    #[zbus(property)]
    fn idle_since_hint(&self) -> zbus::Result<u64>;

    /// IdleSinceHintMonotonic property
    #[zbus(property)]
    fn idle_since_hint_monotonic(&self) -> zbus::Result<u64>;

    /// Leader property
    #[zbus(property)]
    fn leader(&self) -> zbus::Result<u32>;

    /// LockedHint property
    #[zbus(property)]
    fn locked_hint(&self) -> zbus::Result<bool>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// Remote property
    #[zbus(property)]
    fn remote(&self) -> zbus::Result<bool>;

    /// RemoteHost property
    #[zbus(property)]
    fn remote_host(&self) -> zbus::Result<String>;

    /// RemoteUser property
    #[zbus(property)]
    fn remote_user(&self) -> zbus::Result<String>;

    /// Scope property
    #[zbus(property)]
    fn scope(&self) -> zbus::Result<String>;

    /// Seat property
    #[zbus(property)]
    fn seat(&self) -> zbus::Result<(String, zbus::zvariant::OwnedObjectPath)>;

    /// Service property
    #[zbus(property)]
    fn service(&self) -> zbus::Result<String>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<String>;

    /// TTY property
    #[zbus(property, name = "TTY")]
    fn tty(&self) -> zbus::Result<String>;

    /// Timestamp property
    #[zbus(property)]
    fn timestamp(&self) -> zbus::Result<u64>;

    /// TimestampMonotonic property
    #[zbus(property)]
    fn timestamp_monotonic(&self) -> zbus::Result<u64>;

    /// Type property
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;

    /// User property
    #[zbus(property)]
    fn user(&self) -> zbus::Result<(u32, zbus::zvariant::OwnedObjectPath)>;

    /// VTNr property
    #[zbus(property, name = "VTNr")]
    fn vtnr(&self) -> zbus::Result<u32>;
}
