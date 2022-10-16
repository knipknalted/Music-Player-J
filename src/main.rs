// #![windows_subsystem = "windows"]

use lib::spogify::SpogApp;

fn main() {
    let win_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Music Player J", 
        win_options, 
        Box::new(|cc| Box::new(SpogApp::new(cc)))
    )
}

