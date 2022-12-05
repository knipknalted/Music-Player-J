use std::{path::Path, process::Command, time::Duration};

use egui::{pos2, vec2, Color32, FontId};
use rodio::DeviceTrait;

use crate::spogify::{SpogApp, QueueMode, split_artists, SliderMode};
use crate::song::{Song, singleline_galley, init_dir,};
use crate::{WHITE, LIGHT_GREY, SLIDER_BACKGROUND, BLACK};


pub const SPACING: f32 = 35.0;
pub const BUTTON_SIDE: f32 = 40.0;

pub fn settings_window(app: &mut SpogApp, ctx: &egui::Context, frame: &mut eframe::Frame) {
    let height = if app.settings.mini_mode {
        180.0
    } else {
        400.0
    };

    egui::Window::new("Settings").default_height(height).resizable(false).open(&mut app.window_bools.settings).show(ctx, |ui| {
        let directory_error = ui.make_persistent_id("directory_error");
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            // App version
            ui.painter().text(
                ui.max_rect().right_top(), 
                egui::Align2::RIGHT_TOP, 
                env!("CARGO_PKG_VERSION"), 
                egui::FontId::proportional(18.0), 
                ctx.style().visuals.text_color()
            );

            // let currently_reading = ui.make_persistent_id("currently_reading");
            // Label current directory and offer to save if it isn't already saved
            ui.horizontal(|ui| {
                ui.label(&app.dir_path);
                if ui.button("Open in file explorer(WINDOWS ONLY?)").clicked() {
                    Command::new("explorer")
                        .arg(&app.dir_path)
                        .spawn()
                        .unwrap();
                }
                if !app.settings.saved_dirs.contains(&app.dir_path) {
                    if ui.button("Save Directory?").clicked() {
                        app.settings.saved_dirs.push(app.dir_path.clone());
                    }
                }
            });

            // Change directory
            ui.horizontal(|ui| {
                ui.label("New Directory:");
                ui.text_edit_singleline(&mut app.settings.direct_buf);
                let response = ui.button("Confirm new directory path?");
                if response.clicked() {
                    if Path::exists(Path::new(&app.settings.direct_buf)) && !app.song_loading.active {
                        app.dir_path = app.settings.direct_buf.clone();
                        app.current_index = 0;
                    
                        app.track_list = init_dir(&app.dir_path);
                        app.song_loading.request = true;

                    } else if !app.song_loading.active {
                        ui.memory().open_popup(directory_error);
                    }
                }
                egui::popup_below_widget(ui, directory_error, &response, |ui| {
                    ui.label("Couldn't find that directory!");
                });
                egui::popup_below_widget(ui, directory_error, &response, |ui| {
                    ui.label("Please wait until the current directory is finished reading.");
                });
            });

            // Enable/disable fancy rgb color shift
            ui.horizontal(|ui| {
                ui.label("Color Shift");
                let text = match app.settings.color_shift {
                    true => "Turn off".to_string(),
                    false => "Turn on".to_string(),
                };
                if ui.button(text).clicked() {
                    app.settings.color_shift = !app.settings.color_shift;
                    if !app.settings.color_shift {
                        app.color_data.widget_color = Color32::from_rgb(104, 185, 115);
                        app.color_data.widget_detail_color = WHITE;
                    } else {
                        app.color_data.widget_detail_color = SLIDER_BACKGROUND;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Toggle Volume Slider Mode");
                let text = match app.settings.volume_mode {
                    SliderMode::Linear => "Linear".to_string(),
                    SliderMode::Exponential => "Exponential".to_string()
                };
                if ui.button(text).clicked() {
                    match app.settings.volume_mode {
                        SliderMode::Linear => app.settings.volume_mode = SliderMode::Exponential,
                        SliderMode::Exponential => app.settings.volume_mode = SliderMode::Linear,
                    }
                }
            });

            // Misc things for development/debugging
            ui.horizontal(|ui| {
                // Resets the app's output stream, as it's messed up when computer goes to sleep
                if ui.button("Reset Output Stream Handle").clicked() {
                    let (new_stream, new_stream_handle) = rodio::OutputStream::try_default().unwrap();
                    app.stream = new_stream;
                    app.stream_handle = new_stream_handle;
                    app.sink = std::sync::Arc::new(rodio::Sink::try_new(&app.stream_handle).unwrap());
                    app.sink.set_volume(app.volume);
                }
                // Request to refresh metadata without changing directory
                if ui.button("Refresh Metadata").clicked() && !app.song_loading.active {
                    app.song_loading.request = true;
                }
                // Tests fn to split artist string into separate artists
                if ui.button("Print Separated Artists").clicked() {
                    let vec = split_artists(&app.track_list[app.current_index].artist);
                    for artist in vec {
                        println!("{}", artist);
                    }
                }

                // Testing animate_bool
                if ui.button("Play/Pause with fade").clicked() {
                    // let how_on = ui.ctx().animate_bool(egui::Id::null(), app.on);
                    // let how_on = app.on as usize as f32;
                    let sink = app.sink.clone();
                    let volume = app.volume.clone();
                    
                    if app.on {
                        std::thread::spawn(move || {
                            let mut num = 1.0;
                            while num > 0.0 {
                                println!("{num}");
                                std::thread::sleep(Duration::from_millis(10));
                                num -= 0.05;
                                sink.set_volume(volume*num);
                            }
                            sink.pause();
                        });   
                    } else {
                        std::thread::spawn(move || {
                            sink.play();
                            let mut num = 0.0;
                            while num < 1.0 {
                                println!("{num}");
                                std::thread::sleep(Duration::from_millis(10));
                                num += 0.05;
                                sink.set_volume(volume*num);
                            }
                        });
                    }
                    app.on = !app.on;
                }
            });

            // Set volume slider range
            ui.label("Set minimum and maximum values for the volume slider. 1.0 translates to 100% of the computer's volume system. Hard capped to range 0.0 to 1.0");
            ui.horizontal(|ui| {
                ui.label("Min Volume");
                let min = ui.text_edit_singleline(&mut app.settings.volume_buf.0);
                if min.lost_focus() {
                    let new_min = app.settings.volume_buf.0.parse::<f32>();
                    match new_min {
                        Ok(t) => {
                            let num = t.clamp(0.0, app.settings.volume_range.1);
                            app.settings.volume_range.0 = num;
                            app.settings.volume_buf.0 = num.to_string();
                            // println!("num {} volume {}", num, app.volume);
                            if app.volume < num {
                                app.volume = num;
                                app.sink.set_volume(app.volume);
                            }
                        },
                        Err(_) => app.settings.volume_buf.0 = app.settings.volume_range.0.to_string(),
                    }

                }
                let max = ui.text_edit_singleline(&mut app.settings.volume_buf.1);
                if max.lost_focus() {
                    let new_max = app.settings.volume_buf.1.parse::<f32>();
                    match new_max {
                        Ok(t) => {
                            let num = t.clamp(app.settings.volume_range.0, 1.0);
                            app.settings.volume_range.1 = num;
                            app.settings.volume_buf.1 = num.to_string();
                            if app.volume > num {
                                app.volume = num;
                                app.sink.set_volume(app.volume);
                            }
                        },
                        Err(_) => app.settings.volume_buf.1 = app.settings.volume_range.1.to_string(),
                    }

                }
            });
            ui.horizontal(|ui| {
                ui.label("Sink speed");
                if ui.button("0.5").clicked() {
                    app.playback.speed = 0.5;
                    app.sink.set_speed(0.5);
                }
                if ui.button("1.0").clicked() {
                    app.playback.speed = 1.0;
                    app.sink.set_speed(1.0);
                }
                if ui.button("2.0").clicked() {
                    app.playback.speed = 2.0;
                    app.sink.set_speed(2.0);
                }
                if ui.button("4.0").clicked() {
                    app.playback.speed = 4.0;
                    app.sink.set_speed(4.0);
                }
                if ui.button("8.0").clicked() {
                    app.playback.speed = 8.0;
                    app.sink.set_speed(8.0);
                }
                if ui.button("16.0").clicked() {
                    app.playback.speed = 16.0;
                    app.sink.set_speed(16.0);
                }
            });

            ui.separator();

            ui.label("Built-In Downloader");
            ui.horizontal(|ui| {
                ui.label("Downloader Backend Path:");
                ui.text_edit_singleline(&mut app.settings.download_config.dlp_path);
            });
            ui.horizontal(|ui| {
                ui.label("Target folder:");
                ui.text_edit_singleline(&mut app.settings.download_config.target_dir);
            });
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut app.settings.download_config.url);
            });
            ui.horizontal(|ui| {
                ui.label("format:");
                ui.text_edit_singleline(&mut app.settings.download_config.format);
            });
            if ui.button("Try Download ***UNFINISHED").clicked() {
                let bin = app.settings.download_config.dlp_path.as_str();
                let target = app.settings.download_config.target_dir.as_str();
                let url = app.settings.download_config.url.as_str();
                let format = app.settings.download_config.format.as_str();

                let bool = !target.is_empty() && !url.is_empty() && {
                    format == "m4a" || format == "mp3" || format == "flac"
                };
                // ***ADD AND TEST -f ba
                if bool {
                    let arg = format!("-P {} -f ba -x --audio-format {} {}", target, format, url);
                    println!("{}", &arg);
                    Command::new(bin)
                        .arg("-P")
                        .arg(target)
                        .arg("-x")
                        .arg("--audio-format")
                        .arg(format)
                        .arg(url)
                        // .arg(format!("-P {target}"))
                        // .arg("-f ba")
                        // .arg(format!("-x --audio-format {}", format))
                        // .arg(url)
                        .spawn()
                        .expect("Error launching yt-dlp");
                }
            }
            ui.separator();

            // Switch app mode
            if ui.button("Switch App Mode").clicked() {
                app.settings.mini_mode = !app.settings.mini_mode;
                if !app.settings.mini_mode {
                    frame.set_window_size(vec2(800.0, 600.0))
                }
            }

            ui.collapsing("Output Device", |ui| {
                ui.label(format!("Currently using {}", app.devices[app.settings.device_index].name().unwrap()));
                for i in 0..app.devices.len() {
                    if ui.button(app.devices[i].name().unwrap()).clicked() {
                        // Copied from app.set_stream()
                        app.on = false;
                        (app.stream, app.stream_handle) = rodio::OutputStream::try_from_device(&app.devices[i]).unwrap();
                        // app.set_sink();

                        // Copied from app.set_sink()
                        app.sink = std::sync::Arc::new(rodio::Sink::try_new(&app.stream_handle).unwrap());
                        app.sink.set_volume(app.volume);
                        app.sink.set_speed(app.playback.speed)
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Cache directory:");
                let text = if app.settings.cache_dir {
                    "On"
                } else {
                    "Off"
                };
                if ui.button(text).clicked() {
                    app.settings.cache_dir = !app.settings.cache_dir;
                }
            });

            ui.collapsing("Saved Directories", |ui| {
                for i in 0..app.settings.saved_dirs.len() {
                    ui.horizontal(|ui| {
                        if ui.button(&app.settings.saved_dirs[i]).clicked() && !app.song_loading.active {
                            if Path::exists(Path::new(&app.settings.saved_dirs[i])) {
                                // Change the app's directory
                                app.dir_path = app.settings.saved_dirs[i].clone();
                                app.current_index = 0;
                                // Request to refresh the app's track_list
                                app.song_loading.request = true;
                            } else {
                                ui.memory().open_popup(directory_error);
                            }
                        }
                        if ui.button("Remove directory").clicked() {
                            app.settings.saved_dirs.remove(i);
                        }
                        if ui.button("Open in file explorer(WINDOWS ONLY?)").clicked() {
                            Command::new("explorer")
                                .arg(&app.settings.saved_dirs[i])
                                .spawn()
                                .unwrap();
                        }
                    });
                }
            });
        });
        
    });
}

pub fn shuffle_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    let color = app.color_data.widget_color;
    let detail_color = app.color_data.widget_detail_color;

    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));

    let mut response = ui.allocate_rect(rect, egui::Sense::click());
    
    // noting if it's been clicked
    if response.clicked() {
        if matches!(app.mode, QueueMode::Shuffle) {
            app.mode = QueueMode::Next;
        } else {
            app.mode = QueueMode::Shuffle;
        }
        response.mark_changed();
    }
    
    //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));
    // Paint it on the screen
    if ui.is_rect_visible(rect) {
        let mut visuals = ui.style().interact_selectable(&response, true);
        visuals.fg_stroke.width = 1.5;

        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let center = egui::pos2(rect.center().x, rect.center().y);

        ui.painter().rect(rect, radius, color, visuals.fg_stroke);
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER, "🔀", 
            FontId::proportional(rect.height()-4.0), 
            detail_color);
    }
    response
}

pub fn prev(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    // let color = app.color_data.widget_color;
    let detail_color = app.color_data.widget_detail_color;
    
    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));

    let response = ui.allocate_rect(rect, egui::Sense::click());

    // noting if it's been clicked
    if response.clicked() && !app.track_list.is_empty() {
        app.on = true;
        app.go_back(ui.ctx());
        // response.mark_changed();
    }

    //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    //paint it on the screen
    if ui.is_rect_visible(rect) {

        let mut visuals = ui.style().interact_selectable(&response, true);
        visuals.fg_stroke.width = 1.5;

        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let center = egui::pos2(rect.center().x, rect.center().y);

        ui.painter().rect(rect, radius, app.color_data.widget_color, visuals.fg_stroke);
        ui.painter().text(
            center, 
            egui::Align2::CENTER_CENTER, "⏮", 
            FontId::proportional(rect.height()-4.0), 
            detail_color);
    }
    response
}

pub fn play_pause(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    
    // let color = app.color_data.widget_color;
    // let detail_color = app.color_data.widget_detail_color;

    let rect = egui::Rect::from_min_size(rect_nw, 2.0*vec2(BUTTON_SIDE, BUTTON_SIDE));

    // Making space for it
    let mut response = ui.allocate_rect(rect, egui::Sense::click());

    // Noting if it's been clicked
    if response.clicked() {
        response.mark_changed();

        if app.on {
            // std::thread::spawn(move || {
            //     let mut num = 1.0;
            //     while num > 0.0 {
            //         // println!("{num}");
            //         std::thread::sleep(Duration::from_millis(10));
            //         num -= 0.05;
            //         sink.set_volume(volume*num);
            //     }
            //     sink.pause();
            // });

            app.pause();
            println!("Now playing music");
        } else {
            // std::thread::spawn(move || {
            //     sink.play();
            //     let mut num = 0.0;
            //     while num < 1.0 {
            //         // println!("{num}");
            //         std::thread::sleep(Duration::from_millis(10));
            //         num += 0.05;
            //         sink.set_volume(volume*num);
            //     }
            // });

            app.play();
            // app.sink.pause();
            println!("Music paused")
        }
        app.on = !app.on;
    }

    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, app.on, ""));

    // Paint it on the screen
    if ui.is_rect_visible(rect) {

        let mut visuals = ui.style().interact_selectable(&response, app.on);
        visuals.fg_stroke.width = 1.5;

        let color = app.color_data.widget_color;
        let detail_color = app.color_data.widget_detail_color;

        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let center = egui::pos2(rect.center().x, rect.center().y);

        ui.painter().rect(rect, radius, color, visuals.fg_stroke);
        
        if app.on {
            ui.painter().text(
                center, 
                egui::Align2::CENTER_CENTER, "⏸", 
                FontId::proportional(rect.height()-6.0), 
                detail_color);
        } else {
            ui.painter().text(
                center, 
                egui::Align2::CENTER_CENTER, "⏵", 
                FontId::proportional(rect.height()-4.0), 
                detail_color);
        }
    }
    response
}

pub fn next(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    // let color = app.color_data.widget_color;
    let detail_color = app.color_data.widget_detail_color;

    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));

    let response = ui.allocate_rect(rect, egui::Sense::click());

    if response.clicked() && !app.track_list.is_empty() {
        app.skip_song(ui.ctx());
    }

    //paint it on the screen
    if ui.is_rect_visible(rect) {

        let mut visuals = ui.style().interact_selectable(&response, true);
        visuals.fg_stroke.width = 1.5;

        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let center = egui::pos2(rect.center().x, rect.center().y);

        ui.painter().rect(rect, radius, app.color_data.widget_color, visuals.fg_stroke);
        ui.painter().text(
            center, 
            egui::Align2::CENTER_CENTER, "⏭", 
            FontId::proportional(rect.height()-4.0), 
            detail_color);
    }
    response
}

pub fn loop_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    let color = app.color_data.widget_color;
    let detail_color = app.color_data.widget_detail_color;

    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));

    let mut response = ui.allocate_rect(rect, egui::Sense::click());

    if response.clicked() {
        if matches!(app.mode, QueueMode::Loop) {
            app.mode = QueueMode::Next;
        } else {
            app.mode = QueueMode::Loop;
        }
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {

        let mut visuals = ui.style().interact_selectable(&response, true);
        visuals.fg_stroke.width = 1.5;

        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let center = egui::pos2(rect.center().x, rect.center().y);

        ui.painter().rect(rect, radius, color, visuals.fg_stroke);
        ui.painter().text(
            center, 
            egui::Align2::CENTER_CENTER, "🔁", 
            FontId::proportional(rect.height()-4.0), 
            detail_color);
    }
    response
}

pub fn button_decals(mode: &mut QueueMode, ui: &mut egui::Ui, shuffle_nw: egui::Pos2, loop_nw: egui::Pos2) {
    let center = match mode {
        QueueMode::Shuffle => {
            shuffle_nw + vec2(0.5*BUTTON_SIDE, BUTTON_SIDE+10.0)        
        },
        QueueMode::Loop => {
            loop_nw + vec2(0.5*BUTTON_SIDE, BUTTON_SIDE+10.0)
            
        },
        _ => egui::Pos2::ZERO,
    };
    if let QueueMode::Shuffle | QueueMode::Loop = mode {
        ui.painter().circle_filled(center, 5.0, Color32::WHITE);
    }
}

pub fn volume_slider( ui: &mut egui::Ui, app: &mut SpogApp, bar: egui::Rect, min: f32, max: f32) {
    // Slider bar rect defined externally for better control laying out many custom widgets
    
    let center_x = value_to_pos(app.volume, bar.left(), min, max, bar.width(), app.settings.volume_mode);

    let center = pos2(center_x, bar.center().y);
    let knob = egui::Rect::from_center_size(center, vec2(24.0,24.0));

    let knob_response = ui.allocate_rect(knob, egui::Sense::click_and_drag());
    let bar_response = ui.allocate_rect(bar.expand(6.0), egui::Sense {click: true, drag: true, focusable: true});
    if bar_response.is_pointer_button_down_on() {
        let new_pos = bar_response.interact_pointer_pos().unwrap().x;
        app.volume = pos_to_value(new_pos, bar.left(), min, max, bar.width(), app.settings.volume_mode);
        app.sink.set_volume(app.volume);
    }

    if bar_response.hovered() {
        let delta = ui.ctx().input().scroll_delta.y;
        // To save some calculations, don't do if there is no scroll/nothing to change
        if delta != 0.0 {
            let new_pos = knob.center().x + 0.2*delta;
            app.volume = pos_to_value(new_pos, bar.left(), min, max, bar.width(), app.settings.volume_mode);
            if !app.settings.muted {
                app.sink.set_volume(app.volume);
            }
        } 
    }

    // Make this feel more fluid, maybe release if pointer leaves bar rect plus some expansion?
    // Or some way that if pointer is not in bar rect, use it's position
    // This way, dragging it far off the bar to the right, and then slightly to the left, but still outside bar, will not move knob
    if knob_response.dragged() {
        let delta = if ui.rect_contains_pointer(bar_response.rect.expand2(vec2(0.0, 100.0))) {
            knob_response.drag_delta().x
        } else {
            0.0
        };
        let new_pos = knob.center().x + delta;
        app.volume = pos_to_value(new_pos, bar.left(), min, max, bar.width(), app.settings.volume_mode);
        if !app.settings.muted {
            app.sink.set_volume(app.volume);
        }
    }

    // Painting the slider bar
    ui.painter().rect(bar, egui::Rounding::same(10.0), SLIDER_BACKGROUND, egui::Stroke {width: 0.0, color: BLACK});
    // Scuffed speaker icon text
    let text = if app.settings.muted {
        String::from("🔈x")
    } else if app.volume == 0.0 {
        String::from("🔈")
    } else if app.volume<0.5*max {
        String::from("🔉")
    } else {
        String::from("🔊")
    };

    let speaker_rect = ui.painter().text(
        bar.left_center()-vec2(20.0, 0.0), 
        egui::Align2::CENTER_CENTER, 
        text, FontId::proportional(32.0), 
        LIGHT_GREY);
    let speaker_response = ui.allocate_rect(speaker_rect, egui::Sense::click());
    // Simple way to show if pointer is over the mute button (speaker)
    if speaker_response.hovered() {
        let mask = egui::Color32::from_white_alpha(11);
        ui.painter().rect_filled(speaker_rect, egui::Rounding::none(), mask);
    }
 
    if !app.settings.mini_mode {
        let text = match app.settings.volume_mode {
            SliderMode::Linear => "Linear".to_string(),
            SliderMode::Exponential => "Exponential".to_string(),
        };
    
        let mode_gal = ui.painter().layout_no_wrap(text, egui::FontId::proportional(22.0), LIGHT_GREY);
        
        let mode_nw = bar.center_bottom() + vec2(-0.6*mode_gal.rect.width(), 16.0);
        let mode = egui::Rect::from_min_size(mode_nw, vec2(1.6*mode_gal.rect.width(), 24.0));
        let mode_response = ui.allocate_rect(mode, egui::Sense::click());
    
        if mode_response.clicked() {
            match app.settings.volume_mode {
                SliderMode::Linear => app.settings.volume_mode = SliderMode::Exponential,
                SliderMode::Exponential => app.settings.volume_mode = SliderMode::Linear
            }
        }
    
        if ui.is_rect_visible(mode) {
            let mut visuals = ui.style().interact_selectable(&mode_response, true);
            visuals.fg_stroke.width = 1.5;
    
            let mode = mode.expand(visuals.expansion);
    
            ui.painter().rect(mode, 25.0, SLIDER_BACKGROUND, egui::Stroke::new(1.5, LIGHT_GREY));
            ui.painter().galley(mode.center() -vec2(0.5*mode_gal.rect.width(), 0.5*mode_gal.rect.height()), mode_gal);
        }
    }
    
    if speaker_response.clicked() {
        // Switch value of muted
        app.settings.muted = !app.settings.muted;
        // If new value is true, set sink volume to 0 but leave app's stored volume value, if false, set sink to app volume
        if app.settings.muted {
            app.sink.set_volume(0.0)
        } else {
            app.sink.set_volume(app.volume)
        }
    }

    if bar_response.hovered() | knob_response.is_pointer_button_down_on() {
        ui.painter().circle(center, 12.0, app.color_data.widget_color, egui::Stroke {width: 3.0, color: WHITE});
    } else {
        ui.painter().circle_filled(center, 12.0, app.color_data.widget_color);
    }
}

fn pos_to_value(pos: f32, offset: f32, min: f32, max: f32, slide_len: f32, mode: SliderMode) -> f32 {
    let scale_val = ((pos-(offset+12.0))/(slide_len-24.0)).clamp(0.0, 1.0);
    match mode {
        SliderMode::Linear => min +(max-min)*scale_val,
        SliderMode::Exponential => {
            let exp_val = scale_val.powi(3);
            min + (max-min)*exp_val
        },
    }
}

#[inline(always)]
fn value_to_pos(val: f32, offset: f32, min: f32, max: f32, slide_len: f32, mode: SliderMode) -> f32 {
    match mode {
        SliderMode::Linear => (offset+12.0) + ((slide_len-24.0)*(val-min)/(max-min)),
        SliderMode::Exponential => {
            let exp_val = (val-min)/(max+min);
            let lin_val = exp_val.cbrt();
            let val = min + (max-min)*lin_val;
            (offset+12.0) + ((slide_len-24.0)*(val-min)/(max-min))
        }
    }
    
}

pub fn show_current_song(ui: &mut egui::Ui, app: &mut SpogApp, rect: egui::Rect) -> egui::Response {
    // let offset = ui.max_rect().left_center();
    let width = rect.width();
    
    ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
    if app.track_list.len()>0 {
        let song: &Song = &app.track_list[app.current_index];
        // Determine whether there is metadata to display -> use file name if not
        let complete_song = if !song.title.is_empty() && !song.artist.is_empty() && !song.album.is_empty() {
            true
        } else {false};

        use egui::Align;

        if app.settings.mini_mode {
            let left_pos = rect.left_center();
            // if there is some metadata, use it
            if complete_song {
                let right_pos = left_pos + vec2((2.0/3.0)*width, 0.0);
                let (title_gal, title_height) = singleline_galley(
                    ui, width, &song.title.clone(), LIGHT_GREY, 28.0, Align::Min
                );
                let (artist_gal, artist_height) = singleline_galley(
                    ui, width, &song.artist.clone(), LIGHT_GREY, 20.0, Align::Min
                );
                ui.painter().galley(left_pos - vec2(-50.0, 0.5*title_height), title_gal);
                ui.painter().galley(right_pos - vec2(0.0, 0.5*artist_height), artist_gal);
            } else {
                let (name_gal, name_height) = singleline_galley(
                    ui, width, &song.file_name.clone(), LIGHT_GREY, 28.0, Align::Center
                );
                ui.painter().galley(left_pos - vec2(-0.5*width, 0.5*name_height), name_gal);
            }
        } else {
            let top_pos = rect.left_center() + vec2(0.0, -14.0);
            if complete_song {
                let bot_pos = rect.left_center() + vec2(0.0, 16.0);
                let (title_gal, title_height) = singleline_galley(
                    ui, width, &song.title.clone(), LIGHT_GREY, 28.0, Align::Center
                );
                let (artist_gal, artist_height) = singleline_galley(
                    ui, width, &song.artist.clone(), LIGHT_GREY, 20.0, Align::Center
                );
                ui.painter().galley(top_pos - vec2(-0.5*width, 0.5*title_height), title_gal);
                ui.painter().galley(bot_pos - vec2(-0.5*width, 0.5*artist_height), artist_gal);
            } else {
                let (name_gal, name_height) = singleline_galley(
                    ui, width, &song.file_name.clone(), LIGHT_GREY, 28.0, Align::Center
                );
                ui.painter().galley(top_pos - vec2(-0.5*width, 0.5*name_height), name_gal);
            }
        }
    }
    // let response = ui.allocate_rect(rect, egui::Sense::click());
    
    let response = ui.allocate_rect(rect, egui::Sense::click());
    if response.is_pointer_button_down_on() {
        let shade = egui::Color32::from_black_alpha(60);
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), shade);
    }
    response
}

pub fn settings_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) {
    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));
    let cog_response = ui.allocate_rect(rect, egui::Sense::click());

    if cog_response.clicked() {
        println!("woohoo you clicked the button!");
        app.window_bools.settings = !app.window_bools.settings;
    }

    if cog_response.hovered() {
        ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 2.0, color:WHITE});
        ui.painter().text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, "⚙", 
            FontId::proportional(30.0), 
            Color32::WHITE);
    } else {
        ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 1.0, color:LIGHT_GREY});
        ui.painter().text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, "⚙", 
            FontId::proportional(30.0), 
            LIGHT_GREY);
    }         
}

// Toggles whether the app is in full or portable mode
pub fn mode_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2, frame: &mut eframe::Frame) {
    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));
    let response = ui.allocate_rect(rect, egui::Sense::click());
    
    if response.clicked() {
        app.settings.mini_mode = !app.settings.mini_mode;
        if !app.settings.mini_mode {
            frame.set_window_size(vec2(800.0, 600.0))
        }
    }

    let text = if app.settings.mini_mode {
        "⇲".to_string()
    } else {
        "⇱".to_string()
    };

    let color = if response.hovered() {
        WHITE
    } else {
        LIGHT_GREY
    };

    ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 2.0, color});
    ui.painter().text(
        rect.center() - vec2(0.0, 2.0), 
        egui::Align2::CENTER_CENTER, text, 
        FontId::monospace(45.0), 
        color
    );
}

// Width of text in search bar, 
pub fn width_from_str(text: &String, fonts: &egui::text::Fonts, size: f32) -> f32 {
    let mut width = 0.0;
    for char in text.as_str().chars() {
        width += egui::text::Fonts::glyph_width(
            fonts,
            &egui::FontId::proportional(size), 
            char
        );
    }
    width
}

pub fn render_search_bar(ui: &mut egui::Ui, app: &mut SpogApp) {
    let top_left = ui.max_rect().left_top() + vec2(10.0, 5.0);
    let bot_right = top_left + vec2(400.0, 40.0);
    let bar = egui::Rect::from_two_pos(top_left, bot_right);

    // ui.allocate_ui_at_rect(egui::Rect::from_min_size(bar.right_top(), vec2(100.0, 40.0)), |ui| {
    //     let j = match app.filter.selected_j {
    //         Some(u) => u.to_string(),
    //         None => "None".to_string(),
    //     };
    //     ui.label(j);
    //     let i = match app.filter.track_i {
    //         Some(i) => i.to_string(),
    //         None => "None".to_string(),
    //     };
    //     ui.label(i);
    // });

    let magnifying_pos = bar.left_center() + vec2(10.0, 0.0);

    let bar_response = ui.allocate_rect(bar, egui::Sense {click: true, drag: true, focusable: true});
    
    // Accept keyboard input if search bar clicked on
    if bar_response.clicked() {
        app.filter.active = true;
    }
    if bar_response.clicked_elsewhere() {
        app.filter.active = false;
    }
    
    // Clear search bar on right click
    if bar_response.secondary_clicked() {
        app.filter.field = "".to_string();
        app.filter.text_width = 0.0;
        app.filter.active = false;
    }

    let viewport = egui::Rect::from_two_pos(bar.left_top()+vec2(45.0, 0.0), bar.right_bottom()-vec2(20.0,0.0));

    // Horizontal offset for the text if it is longer than the search bar
    let size_shift = if app.filter.text_width < 335.0 {
        0.0
    } else {
        app.filter.text_width-335.0
    };

    // let size_offset = vec2(45.0-size_shift, 0.0);

    // The position used to horizontally offset the search bar text
    let text_pos = viewport.left_center()-vec2(size_shift, 0.0);

    let caret_shift = {
        let mut right_str = app.filter.field.clone();
        right_str.truncate(app.filter.position);
        
        ui.painter().layout(
            right_str, 
            egui::FontId::proportional(28.0), 
            BLACK, 
            f32::INFINITY
        ).rect.width()
    };
    // let mut caret_text_shift = 0.0;
    let caret_x = caret_shift-size_shift;

    if app.filter.active {
        // Handle keyboard input
        for event in &ui.ctx().input().events {
            if let egui::Event::Text(t) = event {
                // let width = width_from_str(t, &app.filter.fonts);
                // app.filter_data.text_width += width;
                app.filter.field.insert_str(app.filter.position, t.as_str());
                app.filter.text_width = width_from_str(&app.filter.field, &app.filter.fonts, 28.0);
                app.filter.position += 1;
                app.filter.blink_timer = std::time::SystemTime::now();
            }
            else if let egui::Event::Key { key, pressed: true, modifiers: _ } = event {
                match key {
                    egui::Key::Enter => {
                        if let Some(i) = app.filter.track_i {
                            app.sink.stop();
                            app.set_sink();
                            app.on = true;
    
                            // Put the previous song into history before playing the one clicked on
                            if app.past_songs.len() == Some(20).unwrap() {
                                app.past_songs.pop_back();
                            }
                            app.past_songs.insert(0, app.current_index);
                            // set the current index to be that of the song clicked on
                            app.current_index = i;
                            app.run_track(ui.ctx());
                        }

                        app.filter.active = false;
                        app.filter.selected_j = None;
                        app.filter.track_i = None;
                    },
                    egui::Key::Backspace => {
                        if app.filter.position != 0 {
                            app.filter.field.remove(app.filter.position-1);
                            // app.filter_data.text_width -= width_from_str(&char, &app.filter_data.fonts);
                            app.filter.text_width = width_from_str(&app.filter.field, &app.filter.fonts, 28.0);
                            app.filter.position -= 1;
                        }
                    },
                    egui::Key::Delete => {
                        if app.filter.position < app.filter.field.chars().count() {
                            app.filter.field.remove(app.filter.position);
                        }
                    },
                    egui::Key::ArrowDown => {
                        if app.filter.selected_j.is_some() {
                            let temp = app.filter.selected_j.unwrap();
                            app.filter.selected_j = Some(temp + 1);
                        } else {
                            app.filter.selected_j = Some(0);
                        }
                    }
                    egui::Key::ArrowUp => {
                        if app.filter.selected_j.is_some() {
                            match app.filter.selected_j.unwrap() {
                                0 => {
                                    app.filter.selected_j = None;
                                    app.filter.track_i = None;
                                },
                                _ => app.filter.selected_j = Some(app.filter.selected_j.unwrap() - 1),
                            }
                        }
                    },
                    egui::Key::ArrowLeft => {
                        if app.filter.position > 0 {
                            app.filter.position -= 1;
                        }
                    },
                    egui::Key::ArrowRight => {
                        if app.filter.position < app.filter.field.chars().count() {
                            app.filter.position += 1;
                        }
                    },
                    _ => ()
                }
                app.filter.blink_timer = std::time::SystemTime::now();
            }
            // let caret_shift = {
            //     let mut right_str = app.filter.field.clone();
            //     right_str.truncate(app.filter.position);
                
            //     ui.painter().layout(
            //         right_str, 
            //         egui::FontId::proportional(28.0), 
            //         BLACK, 
            //         f32::INFINITY
            //     ).rect.width()
            // };

            // let mut caret_x = caret_shift+viewport.left();
        }

        // Painting the bar
        ui.painter().rect_filled(bar, egui::Rounding::same(20.0), WHITE);
        // Painting a decorative magnifying glass
        ui.painter().text(
            magnifying_pos, 
            egui::Align2::LEFT_CENTER, 
            "🔍", egui::FontId::proportional(30.0), 
            BLACK
        );
        
    } else {
        // Activate/ give focus to the search bar if press enter
        for event in &ui.ctx().input().events {
            if let egui::Event::Key {key: egui::Key::Enter, pressed: true, modifiers: _} = event {
                if !app.filter.active {
                    app.filter.active = true;
                }
            }
        }
        // Painting the bar
        ui.painter().rect_filled(bar, egui::Rounding::same(20.0), SLIDER_BACKGROUND);
        // Painting a decorative magnifying glass
        ui.painter().text(
            magnifying_pos,
            egui::Align2::LEFT_CENTER, 
            "🔍", egui::FontId::proportional(30.0), 
            BLACK
        );
    }

    // *** Below is my very clumsy and imperfect method of rendering the text from scratch
    // To determine which part of the text should be shown, if the text is longer than the bar

    

    // temporary button for debugging
    // let test_button = ui.put(egui::Rect::from_min_size(bar.right_top() + vec2(10.0,0.0), vec2(40.0,50.0)), egui::Button::new("test"));
    // if test_button.clicked() {
    //     println!("Bar: {}, Viewport: {}, Text: {}, Caret: {}", bar.left(), viewport.left(), text_pos.x, bar.left()+caret_shift+45.0);
    //     println!("Size shift: {}, Caret shift: {}", size_shift, caret_shift);
    // }  

    // println!("{}", app.filter.position);

    // Hint text if empty search bar
    if app.filter.field.is_empty() {
        ui.painter().with_clip_rect(viewport).text(
            viewport.left_center(), 
            egui::Align2::LEFT_CENTER, 
            "Search by song title, artist, or file address", 
            egui::FontId::proportional(20.0), 
            LIGHT_GREY
        );
    }
    if app.filter.active {
        let test = ui.painter().with_clip_rect(viewport).text(
            text_pos, 
            egui::Align2::LEFT_CENTER, 
            app.filter.field.clone(), 
            egui::FontId::proportional(28.0), 
            BLACK
        );

        // println!("Text: {}, Cached Width: {}, Viewport: {}, Size Shift: {}", test.width(), app.filter.text_width, viewport.right()-viewport.left(), size_shift);
        println!("Text: {}, Cached Width: {}, Diff: {}", test.width(), app.filter.text_width, test.width()-app.filter.text_width);
        let caret_n = viewport.left_top() + vec2(caret_x, 4.0);
        let caret_s = viewport.left_bottom() + vec2(caret_x, -4.0);

        // let caret_n = text_pos + vec2(caret_x-0.5*text_pos.x, -13.0);
        // let caret_s = text_pos + vec2(caret_x-0.5*text_pos.x, 13.0);
        // println!("Caret: {}, Text: {}" caret_n.x, );

        // The blinking caret
        // Arrow keys work, paint it in the right place
        // Now need to 
        // 1) clamp it to the search box bounds
        // 2) adjust the offest for the visible text if the caret would be moved outside it
        let line = egui::epaint::Shape::LineSegment {
            points: [
                caret_n, 
                caret_s
                // bar.left_top()+vec2(test.width(), 5.0)+caret_offset, 
                // bar.left_top()+vec2(test.width(), 35.0)+caret_offset
            ], 
            stroke: egui::Stroke{width: 2.0, color: BLACK}
        };

        // Make caret blink
        ui.ctx().request_repaint();
        if app.filter.blink_timer.elapsed().unwrap().as_secs() % 2 == 0 {
            ui.painter().add(line);
        }
    } else {
        ui.painter().with_clip_rect(viewport).text(
            text_pos, 
            egui::Align2::LEFT_CENTER, 
            app.filter.field.clone(), 
            egui::FontId::proportional(28.0), 
            LIGHT_GREY
        );
    }
}

pub fn progress_bar(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) {
    let slider_len = 300.0;

    let full_bar = egui::Rect::from_min_size(rect_nw, vec2(slider_len, 20.0));
    ui.painter().rect_filled(full_bar, egui::Rounding::same(10.0), SLIDER_BACKGROUND);

    if let Some(duration) = app.track_list[app.current_index].duration {
        println!("hi");
        let millis = duration.as_millis();
        let length = slider_len * (app.playback.elapsed_time as f32/millis as f32);

        let elapsed_bar = egui::Rect::from_min_size(rect_nw, vec2(length, 20.0));
        ui.painter().rect_filled(elapsed_bar, egui::Rounding::same(10.0), WHITE);
    }
}