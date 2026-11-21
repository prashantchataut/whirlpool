//! Power and memory usage (plan section 8).
//!
//! - **EcoQoS** (Windows 11 efficiency mode): the scheduler runs the thread on
//!   efficiency cores at a lower frequency. This suits background dock work
//!   (window tracking and recomposition), but not animations, so it is toggled
//!   as needed.
//! - **Memory trimming**: an idle dock rarely touches most of its pages, so
//!   returning them to the system frees RAM for other apps. If needed again,
//!   they can be restored from the standby list in microseconds.

use std::{
    ffi::c_void,
    mem::size_of,
    sync::atomic::{AtomicU8, Ordering},
};
use windows::Win32::System::{
    ProcessStatus::EmptyWorkingSet,
    SystemServices::GUID_POWER_SAVING_STATUS,
    Threading::{
        GetCurrentProcess, GetCurrentThread, IDLE_PRIORITY_CLASS, MEMORY_PRIORITY_INFORMATION,
        MEMORY_PRIORITY_LOW, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
        ProcessMemoryPriority, ProcessPowerThrottling, SetPriorityClass, SetProcessInformation,
        SetThreadInformation, THREAD_POWER_THROTTLING_CURRENT_VERSION,
        THREAD_POWER_THROTTLING_EXECUTION_SPEED, THREAD_POWER_THROTTLING_STATE,
        ThreadPowerThrottling,
    },
};
use windows::core::GUID;

/// The newer notification distinguishes standard and high-savings modes.
const GUID_ENERGY_SAVER_STATUS: GUID = GUID::from_u128(0x550e8400_e29b_41d4_a716_446655440000);
const UNKNOWN: u8 = 0;
const BATTERY_OFF: u8 = 1;
const BATTERY_ON: u8 = 2;
const ENERGY_OFF: u8 = 3;
const ENERGY_ON: u8 = 4;
static SAVER: SaverState = SaverState(AtomicU8::new(UNKNOWN));

struct SaverState(AtomicU8);

impl SaverState {
    fn update(&self, setting: GUID, value: u32) {
        let next = match (setting, value) {
            (GUID_ENERGY_SAVER_STATUS, 0) => ENERGY_OFF,
            (GUID_ENERGY_SAVER_STATUS, 1 | 2) => ENERGY_ON,
            (GUID_POWER_SAVING_STATUS, 0) => BATTERY_OFF,
            (GUID_POWER_SAVING_STATUS, 1) => BATTERY_ON,
            _ => return,
        };
        // Once the modern source is available, a legacy notification cannot
        // overwrite it. Windows can deliver both to each registered window.
        let _ = self
            .0
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (next >= ENERGY_OFF || current < ENERGY_OFF).then_some(next)
            });
    }

    fn get(&self) -> Option<bool> {
        match self.0.load(Ordering::Relaxed) {
            BATTERY_OFF | ENERGY_OFF => Some(false),
            BATTERY_ON | ENERGY_ON => Some(true),
            _ => None,
        }
    }
}

/// The last state supplied by Windows, or None before the initial notification.
pub(crate) fn notified_saver_state() -> Option<bool> {
    SAVER.get()
}

/// Records the DWORD carried by a PBT_POWERSETTINGCHANGE notification before
/// any redraw is deferred. This preserves the modern energy-saver state
/// instead of replacing it with the legacy battery-saver snapshot.
///
/// # Safety
/// `data` must be the live LPARAM Windows supplied for PBT_POWERSETTINGCHANGE.
/// Its pointed-to POWERBROADCAST_SETTING must remain valid during this call.
pub unsafe fn power_setting_changed(data: windows::Win32::Foundation::LPARAM) {
    // SAFETY: This function's caller guarantees the notification's lifetime.
    if let Some((setting, value)) = unsafe { saver_notification(data) } {
        SAVER.update(setting, value);
    }
}

unsafe fn saver_notification(data: windows::Win32::Foundation::LPARAM) -> Option<(GUID, u32)> {
    use windows::Win32::System::Power::POWERBROADCAST_SETTING;
    if data.0 == 0 {
        return None;
    }
    let setting = data.0 as *const POWERBROADCAST_SETTING;
    // SAFETY: The caller supplies the live broadcast structure. DataLength
    // guards reading the DWORD; read_unaligned handles the packed layout.
    unsafe {
        let guid = std::ptr::addr_of!((*setting).PowerSetting).read_unaligned();
        let size = std::ptr::addr_of!((*setting).DataLength).read_unaligned();
        if size == size_of::<u32>() as u32 {
            let value = std::ptr::addr_of!((*setting).Data)
                .cast::<u32>()
                .read_unaligned();
            return Some((guid, value));
        }
    }
    None
}

pub fn set_thread_efficiency(enabled: bool) {
    let state = THREAD_POWER_THROTTLING_STATE {
        Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: if enabled {
            THREAD_POWER_THROTTLING_EXECUTION_SPEED
        } else {
            0
        },
    };
    // SAFETY: The structure has its size initialized; if unsupported, the call
    // fails without effect.
    unsafe {
        let _ = SetThreadInformation(
            GetCurrentThread(),
            ThreadPowerThrottling,
            (&raw const state).cast::<c_void>(),
            size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

pub fn trim_memory() {
    // SAFETY: Trims this process's own working set.
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Configures a process that only waits (the watchdog) with EcoQoS, minimum
/// CPU priority, and low-priority memory that the system can reclaim first.
pub fn run_as_background_process() {
    let throttling = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    };
    let memory = MEMORY_PRIORITY_INFORMATION {
        MemoryPriority: MEMORY_PRIORITY_LOW,
    };
    // SAFETY: The structures have their sizes initialized and target this
    // process; a failure leaves the default settings unchanged.
    unsafe {
        let process = GetCurrentProcess();
        let _ = SetProcessInformation(
            process,
            ProcessPowerThrottling,
            (&raw const throttling).cast::<c_void>(),
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
        let _ = SetProcessInformation(
            process,
            ProcessMemoryPriority,
            (&raw const memory).cast::<c_void>(),
            size_of::<MEMORY_PRIORITY_INFORMATION>() as u32,
        );
        let _ = SetPriorityClass(process, IDLE_PRIORITY_CLASS);
    }
    trim_memory();
}

/// Tells a window, with `WM_POWERBROADCAST` / `PBT_POWERSETTINGCHANGE`, when
/// energy saver (battery saver on Windows 10) turns on or off: Windows then
/// stops or resumes see-through effects, and glass must follow. Plugging the
/// charger in or out already arrives as `PBT_APMPOWERSTATUSCHANGE`, but
/// turning the saver on by hand does not always.
///
/// Event-driven: nothing runs until Windows sends the change. The
/// subscription ends when this is dropped.
pub struct SaverWatch(Vec<windows::Win32::System::Power::HPOWERNOTIFY>);

impl SaverWatch {
    #[must_use]
    pub fn new(hwnd: isize) -> Self {
        use windows::Win32::{
            Foundation::HANDLE, System::Power::RegisterPowerSettingNotification,
            UI::WindowsAndMessaging::DEVICE_NOTIFY_WINDOW_HANDLE,
        };
        let handles = [GUID_ENERGY_SAVER_STATUS, GUID_POWER_SAVING_STATUS]
            .iter()
            .filter_map(|setting| {
                // SAFETY: `hwnd` is the caller's live window; an unknown
                // setting on an older Windows just fails.
                unsafe {
                    RegisterPowerSettingNotification(
                        HANDLE(hwnd as *mut c_void),
                        setting,
                        DEVICE_NOTIFY_WINDOW_HANDLE,
                    )
                }
                .ok()
            })
            .collect();
        Self(handles)
    }
}

impl Drop for SaverWatch {
    fn drop(&mut self) {
        for handle in self.0.drain(..) {
            // SAFETY: Registered in `new` and released once.
            unsafe {
                let _ = windows::Win32::System::Power::UnregisterPowerSettingNotification(handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saver_notifications_follow_standard_high_savings_and_off() {
        let state = SaverState(AtomicU8::new(UNKNOWN));
        assert_eq!(state.get(), None);
        for (value, enabled) in [(0, false), (1, true), (2, true), (0, false)] {
            state.update(GUID_ENERGY_SAVER_STATUS, value);
            assert_eq!(state.get(), Some(enabled));
        }
    }

    #[test]
    fn legacy_notifications_cannot_override_the_modern_saver_status() {
        let state = SaverState(AtomicU8::new(UNKNOWN));
        state.update(GUID_POWER_SAVING_STATUS, 1);
        assert_eq!(state.get(), Some(true));
        state.update(GUID_POWER_SAVING_STATUS, 0);
        assert_eq!(state.get(), Some(false));
        state.update(GUID_ENERGY_SAVER_STATUS, 1);
        state.update(GUID_POWER_SAVING_STATUS, 0);
        assert_eq!(state.get(), Some(true));
        state.update(GUID_ENERGY_SAVER_STATUS, 0);
        state.update(GUID_POWER_SAVING_STATUS, 1);
        assert_eq!(state.get(), Some(false));
    }

    #[test]
    fn unknown_guids_and_values_preserve_the_last_known_status() {
        let state = SaverState(AtomicU8::new(UNKNOWN));
        state.update(GUID::zeroed(), 1);
        state.update(GUID_ENERGY_SAVER_STATUS, 99);
        assert_eq!(state.get(), None);
        state.update(GUID_ENERGY_SAVER_STATUS, 1);
        state.update(GUID_POWER_SAVING_STATUS, 99);
        assert_eq!(state.get(), Some(true));
    }

    #[test]
    fn notification_reader_checks_the_length_and_reads_the_dword() {
        use windows::Win32::{Foundation::LPARAM, System::Power::POWERBROADCAST_SETTING};
        #[repr(C)]
        struct Message {
            setting: GUID,
            length: u32,
            value: u32,
        }
        let message = Message {
            setting: GUID_ENERGY_SAVER_STATUS,
            length: 4,
            value: 2,
        };
        let data = LPARAM((&raw const message).cast::<POWERBROADCAST_SETTING>() as isize);
        // SAFETY: The fixture includes the complete header and DWORD payload.
        assert_eq!(
            unsafe { saver_notification(data) },
            Some((GUID_ENERGY_SAVER_STATUS, 2))
        );
        let short = Message {
            length: 1,
            ..message
        };
        let data = LPARAM((&raw const short).cast::<POWERBROADCAST_SETTING>() as isize);
        // SAFETY: The fixture remains live and contains the complete header.
        assert_eq!(unsafe { saver_notification(data) }, None);
        // SAFETY: Null is explicitly accepted as an absent notification.
        assert_eq!(unsafe { saver_notification(LPARAM(0)) }, None);
    }
}
