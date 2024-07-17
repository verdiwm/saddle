//! # D-Bus interface proxy for: `org.freedesktop.login1.Manager`
use zbus::{proxy, Result};

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Manager {
    /// ActivateSession method
    fn activate_session(&self, session_id: &str) -> Result<()>;

    /// ActivateSessionOnSeat method
    fn activate_session_on_seat(&self, session_id: &str, seat_id: &str) -> Result<()>;

    /// AttachDevice method
    fn attach_device(&self, seat_id: &str, sysfs_path: &str, interactive: bool) -> Result<()>;

    /// CanHalt method
    fn can_halt(&self) -> Result<String>;

    /// CanHibernate method
    fn can_hibernate(&self) -> Result<String>;

    /// CanHybridSleep method
    fn can_hybrid_sleep(&self) -> Result<String>;

    /// CanPowerOff method
    fn can_power_off(&self) -> Result<String>;

    /// CanReboot method
    fn can_reboot(&self) -> Result<String>;

    /// CanRebootParameter method
    fn can_reboot_parameter(&self) -> Result<String>;

    /// CanRebootToBootLoaderEntry method
    fn can_reboot_to_boot_loader_entry(&self) -> Result<String>;

    /// CanRebootToBootLoaderMenu method
    fn can_reboot_to_boot_loader_menu(&self) -> Result<String>;

    /// CanRebootToFirmwareSetup method
    fn can_reboot_to_firmware_setup(&self) -> Result<String>;

    /// CanSleep method
    fn can_sleep(&self) -> Result<String>;

    /// CanSuspend method
    fn can_suspend(&self) -> Result<String>;

    /// CanSuspendThenHibernate method
    fn can_suspend_then_hibernate(&self) -> Result<String>;

    /// CancelScheduledShutdown method
    fn cancel_scheduled_shutdown(&self) -> Result<bool>;

    /// CreateSession method
    #[allow(clippy::too_many_arguments)]
    fn create_session(
        &self,
        uid: u32,
        pid: u32,
        service: &str,
        type_: &str,
        class: &str,
        desktop: &str,
        seat_id: &str,
        vtnr: u32,
        tty: &str,
        display: &str,
        remote: bool,
        remote_user: &str,
        remote_host: &str,
        properties: &[&(&str, &zbus::zvariant::Value<'_>)],
    ) -> Result<(
        String,
        zbus::zvariant::OwnedObjectPath,
        String,
        zbus::zvariant::OwnedFd,
        u32,
        String,
        u32,
        bool,
    )>;

    /// CreateSessionWithPIDFD method
    #[zbus(name = "CreateSessionWithPIDFD")]
    #[allow(clippy::too_many_arguments)]
    fn create_session_with_pidfd(
        &self,
        uid: u32,
        pidfd: zbus::zvariant::Fd<'_>,
        service: &str,
        type_: &str,
        class: &str,
        desktop: &str,
        seat_id: &str,
        vtnr: u32,
        tty: &str,
        display: &str,
        remote: bool,
        remote_user: &str,
        remote_host: &str,
        flags: u64,
        properties: &[&(&str, &zbus::zvariant::Value<'_>)],
    ) -> Result<(
        String,
        zbus::zvariant::OwnedObjectPath,
        String,
        zbus::zvariant::OwnedFd,
        u32,
        String,
        u32,
        bool,
    )>;

    /// FlushDevices method
    fn flush_devices(&self, interactive: bool) -> Result<()>;

    /// GetSeat method
    fn get_seat(&self, seat_id: &str) -> Result<zbus::zvariant::OwnedObjectPath>;

    /// GetSession method
    fn get_session(&self, session_id: &str) -> Result<zbus::zvariant::OwnedObjectPath>;

    /// GetSessionByPID method
    #[zbus(name = "GetSessionByPID")]
    fn get_session_by_pid(&self, pid: u32) -> Result<zbus::zvariant::OwnedObjectPath>;

    /// GetUser method
    fn get_user(&self, uid: u32) -> Result<zbus::zvariant::OwnedObjectPath>;

    /// GetUserByPID method
    #[zbus(name = "GetUserByPID")]
    fn get_user_by_pid(&self, pid: u32) -> Result<zbus::zvariant::OwnedObjectPath>;

    /// Halt method
    fn halt(&self, interactive: bool) -> Result<()>;

    /// HaltWithFlags method
    fn halt_with_flags(&self, flags: u64) -> Result<()>;

    /// Hibernate method
    fn hibernate(&self, interactive: bool) -> Result<()>;

    /// HibernateWithFlags method
    fn hibernate_with_flags(&self, flags: u64) -> Result<()>;

    /// HybridSleep method
    fn hybrid_sleep(&self, interactive: bool) -> Result<()>;

    /// HybridSleepWithFlags method
    fn hybrid_sleep_with_flags(&self, flags: u64) -> Result<()>;

    /// Inhibit method
    fn inhibit(
        &self,
        what: &str,
        who: &str,
        why: &str,
        mode: &str,
    ) -> Result<zbus::zvariant::OwnedFd>;

    /// KillSession method
    fn kill_session(&self, session_id: &str, who: &str, signal_number: i32) -> Result<()>;

    /// KillUser method
    fn kill_user(&self, uid: u32, signal_number: i32) -> Result<()>;

    /// ListInhibitors method
    fn list_inhibitors(&self) -> Result<Vec<(String, String, String, String, u32, u32)>>;

    /// ListSeats method
    fn list_seats(&self) -> Result<Vec<(String, zbus::zvariant::OwnedObjectPath)>>;

    /// ListSessions method
    fn list_sessions(
        &self,
    ) -> Result<Vec<(String, u32, String, String, zbus::zvariant::OwnedObjectPath)>>;

    /// ListSessionsEx method
    fn list_sessions_ex(
        &self,
    ) -> Result<
        Vec<(
            String,
            u32,
            String,
            String,
            u32,
            String,
            String,
            bool,
            u64,
            zbus::zvariant::OwnedObjectPath,
        )>,
    >;

    /// ListUsers method
    fn list_users(&self) -> Result<Vec<(u32, String, zbus::zvariant::OwnedObjectPath)>>;

    /// LockSession method
    fn lock_session(&self, session_id: &str) -> Result<()>;

    /// LockSessions method
    fn lock_sessions(&self) -> Result<()>;

    /// PowerOff method
    fn power_off(&self, interactive: bool) -> Result<()>;

    /// PowerOffWithFlags method
    fn power_off_with_flags(&self, flags: u64) -> Result<()>;

    /// Reboot method
    fn reboot(&self, interactive: bool) -> Result<()>;

    /// RebootWithFlags method
    fn reboot_with_flags(&self, flags: u64) -> Result<()>;

    /// ReleaseSession method
    fn release_session(&self, session_id: &str) -> Result<()>;

    /// ScheduleShutdown method
    fn schedule_shutdown(&self, type_: &str, usec: u64) -> Result<()>;

    /// SetRebootParameter method
    fn set_reboot_parameter(&self, parameter: &str) -> Result<()>;

    /// SetRebootToBootLoaderEntry method
    fn set_reboot_to_boot_loader_entry(&self, boot_loader_entry: &str) -> Result<()>;

    /// SetRebootToBootLoaderMenu method
    fn set_reboot_to_boot_loader_menu(&self, timeout: u64) -> Result<()>;

    /// SetRebootToFirmwareSetup method
    fn set_reboot_to_firmware_setup(&self, enable: bool) -> Result<()>;

    /// SetUserLinger method
    fn set_user_linger(&self, uid: u32, enable: bool, interactive: bool) -> Result<()>;

    /// SetWallMessage method
    fn set_wall_message(&self, wall_message: &str, enable: bool) -> Result<()>;

    /// Sleep method
    fn sleep(&self, flags: u64) -> Result<()>;

    /// Suspend method
    fn suspend(&self, interactive: bool) -> Result<()>;

    /// SuspendThenHibernate method
    fn suspend_then_hibernate(&self, interactive: bool) -> Result<()>;

    /// SuspendThenHibernateWithFlags method
    fn suspend_then_hibernate_with_flags(&self, flags: u64) -> Result<()>;

    /// SuspendWithFlags method
    fn suspend_with_flags(&self, flags: u64) -> Result<()>;

    /// TerminateSeat method
    fn terminate_seat(&self, seat_id: &str) -> Result<()>;

    /// TerminateSession method
    fn terminate_session(&self, session_id: &str) -> Result<()>;

    /// TerminateUser method
    fn terminate_user(&self, uid: u32) -> Result<()>;

    /// UnlockSession method
    fn unlock_session(&self, session_id: &str) -> Result<()>;

    /// UnlockSessions method
    fn unlock_sessions(&self) -> Result<()>;

    /// PrepareForShutdown signal
    #[zbus(signal)]
    fn prepare_for_shutdown(&self, start: bool) -> Result<()>;

    /// PrepareForShutdownWithMetadata signal
    #[zbus(signal)]
    fn prepare_for_shutdown_with_metadata(
        &self,
        start: bool,
        metadata: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> Result<()>;

    /// PrepareForSleep signal
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool) -> Result<()>;

    /// SeatNew signal
    #[zbus(signal)]
    fn seat_new(&self, seat_id: &str, object_path: zbus::zvariant::ObjectPath<'_>) -> Result<()>;

    /// SeatRemoved signal
    #[zbus(signal)]
    fn seat_removed(
        &self,
        seat_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> Result<()>;

    /// SessionNew signal
    #[zbus(signal)]
    fn session_new(
        &self,
        session_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> Result<()>;

    /// SessionRemoved signal
    #[zbus(signal)]
    fn session_removed(
        &self,
        session_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> Result<()>;

    /// UserNew signal
    #[zbus(signal)]
    fn user_new(&self, uid: u32, object_path: zbus::zvariant::ObjectPath<'_>) -> Result<()>;

    /// UserRemoved signal
    #[zbus(signal)]
    fn user_removed(&self, uid: u32, object_path: zbus::zvariant::ObjectPath<'_>) -> Result<()>;

    /// BlockInhibited property
    #[zbus(property)]
    fn block_inhibited(&self) -> Result<String>;

    /// BootLoaderEntries property
    #[zbus(property)]
    fn boot_loader_entries(&self) -> Result<Vec<String>>;

    /// DelayInhibited property
    #[zbus(property)]
    fn delay_inhibited(&self) -> Result<String>;

    /// Docked property
    #[zbus(property)]
    fn docked(&self) -> Result<bool>;

    /// EnableWallMessages property
    #[zbus(property)]
    fn enable_wall_messages(&self) -> Result<bool>;
    #[zbus(property)]
    fn set_enable_wall_messages(&self, value: bool) -> Result<()>;

    /// HandleHibernateKey property
    #[zbus(property)]
    fn handle_hibernate_key(&self) -> Result<String>;

    /// HandleHibernateKeyLongPress property
    #[zbus(property)]
    fn handle_hibernate_key_long_press(&self) -> Result<String>;

    /// HandleLidSwitch property
    #[zbus(property)]
    fn handle_lid_switch(&self) -> Result<String>;

    /// HandleLidSwitchDocked property
    #[zbus(property)]
    fn handle_lid_switch_docked(&self) -> Result<String>;

    /// HandleLidSwitchExternalPower property
    #[zbus(property)]
    fn handle_lid_switch_external_power(&self) -> Result<String>;

    /// HandlePowerKey property
    #[zbus(property)]
    fn handle_power_key(&self) -> Result<String>;

    /// HandlePowerKeyLongPress property
    #[zbus(property)]
    fn handle_power_key_long_press(&self) -> Result<String>;

    /// HandleRebootKey property
    #[zbus(property)]
    fn handle_reboot_key(&self) -> Result<String>;

    /// HandleRebootKeyLongPress property
    #[zbus(property)]
    fn handle_reboot_key_long_press(&self) -> Result<String>;

    /// HandleSuspendKey property
    #[zbus(property)]
    fn handle_suspend_key(&self) -> Result<String>;

    /// HandleSuspendKeyLongPress property
    #[zbus(property)]
    fn handle_suspend_key_long_press(&self) -> Result<String>;

    /// HoldoffTimeoutUSec property
    #[zbus(property, name = "HoldoffTimeoutUSec")]
    fn holdoff_timeout_usec(&self) -> Result<u64>;

    /// IdleAction property
    #[zbus(property)]
    fn idle_action(&self) -> Result<String>;

    /// IdleActionUSec property
    #[zbus(property, name = "IdleActionUSec")]
    fn idle_action_usec(&self) -> Result<u64>;

    /// IdleHint property
    #[zbus(property)]
    fn idle_hint(&self) -> Result<bool>;

    /// IdleSinceHint property
    #[zbus(property)]
    fn idle_since_hint(&self) -> Result<u64>;

    /// IdleSinceHintMonotonic property
    #[zbus(property)]
    fn idle_since_hint_monotonic(&self) -> Result<u64>;

    /// InhibitDelayMaxUSec property
    #[zbus(property, name = "InhibitDelayMaxUSec")]
    fn inhibit_delay_max_usec(&self) -> Result<u64>;

    /// InhibitorsMax property
    #[zbus(property)]
    fn inhibitors_max(&self) -> Result<u64>;

    /// KillExcludeUsers property
    #[zbus(property)]
    fn kill_exclude_users(&self) -> Result<Vec<String>>;

    /// KillOnlyUsers property
    #[zbus(property)]
    fn kill_only_users(&self) -> Result<Vec<String>>;

    /// KillUserProcesses property
    #[zbus(property)]
    fn kill_user_processes(&self) -> Result<bool>;

    /// LidClosed property
    #[zbus(property)]
    fn lid_closed(&self) -> Result<bool>;

    /// NAutoVTs property
    #[zbus(property, name = "NAutoVTs")]
    fn nauto_vts(&self) -> Result<u32>;

    /// NCurrentInhibitors property
    #[zbus(property, name = "NCurrentInhibitors")]
    fn ncurrent_inhibitors(&self) -> Result<u64>;

    /// NCurrentSessions property
    #[zbus(property, name = "NCurrentSessions")]
    fn ncurrent_sessions(&self) -> Result<u64>;

    /// OnExternalPower property
    #[zbus(property)]
    fn on_external_power(&self) -> Result<bool>;

    /// PreparingForShutdown property
    #[zbus(property)]
    fn preparing_for_shutdown(&self) -> Result<bool>;

    /// PreparingForSleep property
    #[zbus(property)]
    fn preparing_for_sleep(&self) -> Result<bool>;

    /// RebootParameter property
    #[zbus(property)]
    fn reboot_parameter(&self) -> Result<String>;

    /// RebootToBootLoaderEntry property
    #[zbus(property)]
    fn reboot_to_boot_loader_entry(&self) -> Result<String>;

    /// RebootToBootLoaderMenu property
    #[zbus(property)]
    fn reboot_to_boot_loader_menu(&self) -> Result<u64>;

    /// RebootToFirmwareSetup property
    #[zbus(property)]
    fn reboot_to_firmware_setup(&self) -> Result<bool>;

    /// RemoveIPC property
    #[zbus(property, name = "RemoveIPC")]
    fn remove_ipc(&self) -> Result<bool>;

    /// RuntimeDirectoryInodesMax property
    #[zbus(property)]
    fn runtime_directory_inodes_max(&self) -> Result<u64>;

    /// RuntimeDirectorySize property
    #[zbus(property)]
    fn runtime_directory_size(&self) -> Result<u64>;

    /// ScheduledShutdown property
    #[zbus(property)]
    fn scheduled_shutdown(&self) -> Result<(String, u64)>;

    /// SessionsMax property
    #[zbus(property)]
    fn sessions_max(&self) -> Result<u64>;

    /// SleepOperation property
    #[zbus(property)]
    fn sleep_operation(&self) -> Result<Vec<String>>;

    /// StopIdleSessionUSec property
    #[zbus(property, name = "StopIdleSessionUSec")]
    fn stop_idle_session_usec(&self) -> Result<u64>;

    /// UserStopDelayUSec property
    #[zbus(property, name = "UserStopDelayUSec")]
    fn user_stop_delay_usec(&self) -> Result<u64>;

    /// WallMessage property
    #[zbus(property)]
    fn wall_message(&self) -> Result<String>;
    // FIXME: this causes a name collision without the _ suffix
    #[zbus(property)]
    fn set_wall_message_(&self, value: &str) -> Result<()>;
}
