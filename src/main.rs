// #![windows_subsystem = "windows"]

use lib::spogify::SpogApp;

fn main() {
    // let profiler = dhat::Profiler::new_heap(); 
    // options for launching the app
    // let win_options = eframe::NativeOptions::default(); //{default_theme: eframe::Theme::Dark, ..Default::default()};
    let win_options = eframe::NativeOptions {
        // min_window_size: Some(egui::vec2(800.0, 600.0)), 
        // hardware_acceleration: eframe::HardwareAcceleration::Off, 
        ..Default::default()};
    // path used to initialize the app   
    // let dir_path = r"D:\Music\playlist\music";
    eframe::run_native(
        "Spogify", 
        win_options, 
        Box::new(|cc| Box::new(SpogApp::new(cc))))
}

