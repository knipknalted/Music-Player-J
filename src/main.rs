#![windows_subsystem = "windows"]

use lib::spogify::SpogApp;
// use windows_sys::core::GUID;

// const GUID_MONITOR_POWER_ON: GUID = windows_sys::Win32::System::SystemServices::GUID_MONITOR_POWER_ON;

fn main() {
    // let power_notify = unsafe { 
    //     windows_sys::Win32::System::Power::RegisterPowerSettingNotification(
    //         windows_sys::Win32::System::Threading::GetCurrentProcess(), 
    //         &GUID_MONITOR_POWER_ON as *const GUID, 
    //         0
    //     ) 
    // };
    let win_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Music Player J", 
        win_options, 
        Box::new(|cc| Box::new(SpogApp::new(cc)))
    )
}

