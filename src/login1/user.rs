//! # D-Bus interface proxy for: `org.freedesktop.login1.User`
use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.login1.User",
    default_service = "org.freedesktop.login1"
)]
trait User {
    /// Kill method
    fn kill(&self, signal_number: i32) -> zbus::Result<()>;

    /// Terminate method
    fn terminate(&self) -> zbus::Result<()>;

    /// Display property
    #[zbus(property)]
    fn display(&self) -> zbus::Result<(String, zbus::zvariant::OwnedObjectPath)>;

    /// GID property
    #[zbus(property, name = "GID")]
    fn gid(&self) -> zbus::Result<u32>;

    /// IdleHint property
    #[zbus(property)]
    fn idle_hint(&self) -> zbus::Result<bool>;

    /// IdleSinceHint property
    #[zbus(property)]
    fn idle_since_hint(&self) -> zbus::Result<u64>;

    /// IdleSinceHintMonotonic property
    #[zbus(property)]
    fn idle_since_hint_monotonic(&self) -> zbus::Result<u64>;

    /// Linger property
    #[zbus(property)]
    fn linger(&self) -> zbus::Result<bool>;

    /// Name property
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    /// RuntimePath property
    #[zbus(property)]
    fn runtime_path(&self) -> zbus::Result<String>;

    /// Service property
    #[zbus(property)]
    fn service(&self) -> zbus::Result<String>;

    /// Sessions property
    #[zbus(property)]
    fn sessions(&self) -> zbus::Result<Vec<(String, zbus::zvariant::OwnedObjectPath)>>;

    /// Slice property
    #[zbus(property)]
    fn slice(&self) -> zbus::Result<String>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<String>;

    /// Timestamp property
    #[zbus(property)]
    fn timestamp(&self) -> zbus::Result<u64>;

    /// TimestampMonotonic property
    #[zbus(property)]
    fn timestamp_monotonic(&self) -> zbus::Result<u64>;

    /// UID property
    #[zbus(property, name = "UID")]
    fn uid(&self) -> zbus::Result<u32>;
}
