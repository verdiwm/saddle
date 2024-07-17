//! # D-Bus interface proxy for: `org.freedesktop.login1.Seat`
use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.login1.Seat",
    default_service = "org.freedesktop.login1",
)]
trait Seat {
    /// ActivateSession method
    fn activate_session(&self, session_id: &str) -> zbus::Result<()>;

    /// SwitchTo method
    fn switch_to(&self, vtnr: u32) -> zbus::Result<()>;

    /// SwitchToNext method
    fn switch_to_next(&self) -> zbus::Result<()>;

    /// SwitchToPrevious method
    fn switch_to_previous(&self) -> zbus::Result<()>;

    /// Terminate method
    fn terminate(&self) -> zbus::Result<()>;

    /// ActiveSession property
    #[zbus(property)]
    fn active_session(&self) -> zbus::Result<(String, zbus::zvariant::OwnedObjectPath)>;

    /// CanGraphical property
    #[zbus(property)]
    fn can_graphical(&self) -> zbus::Result<bool>;

    /// CanTTY property
    #[zbus(property, name = "CanTTY")]
    fn can_tty(&self) -> zbus::Result<bool>;

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

    /// Sessions property
    #[zbus(property)]
    fn sessions(&self) -> zbus::Result<Vec<(String, zbus::zvariant::OwnedObjectPath)>>;
}
