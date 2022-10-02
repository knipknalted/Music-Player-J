use std::{path::Path};

use egui::{pos2};
use epaint::{Color32, vec2, FontId};

use crate::spogify::{SpogApp, QueueState, get_tracks, get_song, singleline_galley, Song};

pub const WHITE: egui::Color32 = egui::Color32::WHITE;
pub const BLACK: egui::Color32 = egui::Color32::BLACK;
pub const LIGHT_GREY: egui::Color32 = egui::Color32::from_rgb(167, 167, 167);
pub const _BACKGROUND: egui::Color32 = egui::Color32::from_rgb(27, 27, 27);
pub const SLIDER_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(60, 60, 60);
pub const _FERN: Color32 = Color32::from_rgb(104, 185, 115);
pub const _CUSTOM_BLUE: Color32 = Color32::from_rgb(132,150,255);
pub const SPACING: f32 = 35.0;
pub const BUTTON_SIDE: f32 = 40.0;

// pub fn shuffle_button(ui: &mut egui::Ui, app: &mut SpogApp, available_rect: egui::Rect) -> egui::Response {
//     let rect_center = available_rect.center();
//     let color = app.color_data.widget_color;
//     let detail_color = app.color_data.widget_detail_color;
    
//     let h_offset = vec2(2.5 * SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
//     let rect = egui::Rect::from_min_max(
//         rect_center - h_offset - vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y),
//         rect_center - h_offset + vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y));

//     let mut response = ui.allocate_rect(rect, egui::Sense::click());
    
//     // noting if it's been clicked
//     if response.clicked() {
//         if matches!(app.mode, QueueState::Shuffle) {
//             app.mode = QueueState::Next;
//         } else {
//             app.mode = QueueState::Shuffle;
//         }
//         response.mark_changed();
//     }
    
//     //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));
//     // Paint it on the screen
//     if ui.is_rect_visible(rect) {
//         let mut visuals = ui.style().interact_selectable(&response, true);
//         visuals.fg_stroke.width = 1.5;

//         let rect = rect.expand(visuals.expansion);
//         let radius = 0.5 * rect.height();
//         let center = egui::pos2(rect.center().x, rect.center().y);

//         ui.painter().rect(rect, radius, color, visuals.fg_stroke);
//         ui.painter().text(
//             center,
//             egui::Align2::CENTER_CENTER, "🔀", 
//             FontId::proportional(rect.height()-4.0), 
//             detail_color);
//     }
//     response
// }

// pub fn prev(ui: &mut egui::Ui, app: &mut SpogApp, available_rect: egui::Rect) -> egui::Response {
//     let rect_center = available_rect.center();
//     // let color = app.color_data.widget_color;
//     let detail_color = app.color_data.widget_detail_color;
    
//     let h_offset = vec2(SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
//     let rect = egui::Rect::from_min_max(
//         rect_center - h_offset - vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y),
//         rect_center - h_offset + vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y));

//     let response = ui.allocate_rect(rect, egui::Sense::click());

//     // noting if it's been clicked
//     if response.clicked() {
//         app.go_back();
//         // response.mark_changed();
//     }
//     //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

//     //paint it on the screen
//     if ui.is_rect_visible(rect) {

//         let mut visuals = ui.style().interact_selectable(&response, true);
//         visuals.fg_stroke.width = 1.5;

//         let rect = rect.expand(visuals.expansion);
//         let radius = 0.5 * rect.height();
//         let center = egui::pos2(rect.center().x, rect.center().y);

//         ui.painter().rect(rect, radius, app.color_data.widget_color, visuals.fg_stroke);
//         ui.painter().text(
//             center, 
//             egui::Align2::CENTER_CENTER, "⏮", 
//             FontId::proportional(rect.height()-4.0), 
//             detail_color);
//     }
//     response
// }

// pub fn play_pause(ui: &mut egui::Ui, app: &mut SpogApp, available_rect: egui::Rect) -> egui::Response {
//     let rect_center = available_rect.center();
//     let rect = egui::Rect::from_min_max(
//         rect_center - 2.0 * pos2(ui.spacing().interact_size.y, ui.spacing().interact_size.y).to_vec2(),
//         rect_center + 2.0 * pos2(ui.spacing().interact_size.y, ui.spacing().interact_size.y).to_vec2());
//     // setting size
//     //println!("{} to {}, {} to {}, size {}",rect.left(), rect.right(), rect.top(), rect.bottom(), ui.spacing().interact_size.y);
//     //let desired_size = ui.spacing().interact_size.y * egui::vec2(4.0,4.0);
//     //println!("{}", rect.height());

//     // making space for it
//     let mut response = ui.allocate_rect(rect, egui::Sense::click());

//     // noting if it's been clicked
//     if response.clicked() {
//         app.on = !app.on;
//         response.mark_changed();
//         if app.on {
//             app.sink.play();
//         } else { 
//             app.sink.pause();
//         }
//     }

//     response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, app.on, ""));

//     //paint it on the screen
//     if ui.is_rect_visible(rect) {

//         let mut visuals = ui.style().interact_selectable(&response, app.on);
//         visuals.fg_stroke.width = 1.5;

//         let color = app.color_data.widget_color;
//         let detail_color = app.color_data.widget_detail_color;

//         let rect = rect.expand(visuals.expansion);
//         let radius = 0.5 * rect.height();
//         let center = egui::pos2(rect.center().x, rect.center().y);

//         ui.painter().rect(rect, radius, color, visuals.fg_stroke);

//         let pause_rects = vec![
//             epaint::Rect::from_two_pos(pos2(center.x-0.2*rect.width(),center.y-0.2*rect.height()), 
//                 pos2(center.x-0.1*rect.width(),center.y+0.2*rect.height())),
//             egui::Rect::from_two_pos(pos2(center.x+0.2*rect.width(),center.y-0.2*rect.height()), 
//                 pos2(center.x+0.1*rect.width(),center.y+0.2*rect.height()))
//             ];
//         let _pause: Vec<epaint::Shape> = vec![
//             egui::Shape::Rect(epaint::RectShape {
//                 rect: pause_rects[0], 
//                 rounding: epaint::Rounding::none(), 
//                 fill: epaint::Color32::WHITE, 
//                 stroke: visuals.fg_stroke}),
//             egui::Shape::Rect(epaint::RectShape {
//                 rect: pause_rects[1], 
//                 rounding: epaint::Rounding::none(), 
//                 fill: epaint::Color32::WHITE, 
//                 stroke: visuals.fg_stroke})
//                 ];

//         let _play = epaint::PathShape {
//             points: vec![pos2(center.x-rect.width()*0.175, center.y-rect.height()*0.233333),
//                         pos2(center.x-rect.width()*0.175, center.y+rect.height()*0.233333),
//                         pos2(center.x+rect.width()*0.29, center.y)],
//             closed: true,
//             fill: epaint::Color32::WHITE,
//             stroke: visuals.fg_stroke,

//         };
        
//         if app.on {
//             //ui.painter().extend(pause);
//             ui.painter().text(
//                 center, 
//                 egui::Align2::CENTER_CENTER, "⏸", 
//                 FontId::proportional(rect.height()-6.0), 
//                 detail_color);
//         } else {
//             //ui.painter().add(play); ▶
//             ui.painter().text(
//                 center, 
//                 egui::Align2::CENTER_CENTER, "⏵", 
//                 FontId::proportional(rect.height()-4.0), 
//                 detail_color);
//         }
//     }
//     response
// }

// pub fn next(ui: &mut egui::Ui, app: &mut SpogApp, available_rect: egui::Rect) -> egui::Response {
//     let rect_center = available_rect.center();
//     // let color = app.color_data.widget_color;
//     let detail_color = app.color_data.widget_detail_color;
    
//     let h_offset = vec2(SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
//     let rect = egui::Rect::from_min_max(
//         rect_center + h_offset - vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y),
//         rect_center + h_offset + vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y));

//     let response = ui.allocate_rect(rect, egui::Sense::click());

//     // noting if it's been clicked
//     if response.clicked() {
//         app.skip_song();
//         // response.mark_changed();
//     }

//     //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

//     //paint it on the screen
//     if ui.is_rect_visible(rect) {

//         let mut visuals = ui.style().interact_selectable(&response, true);
//         visuals.fg_stroke.width = 1.5;

//         let rect = rect.expand(visuals.expansion);
//         let radius = 0.5 * rect.height();
//         let center = egui::pos2(rect.center().x, rect.center().y);

//         ui.painter().rect(rect, radius, app.color_data.widget_color, visuals.fg_stroke);
//         ui.painter().text(
//             center, 
//             egui::Align2::CENTER_CENTER, "⏭", 
//             FontId::proportional(rect.height()-4.0), 
//             detail_color);
//     }
//     response
// }

// pub fn loop_button(ui: &mut egui::Ui, app: &mut SpogApp, available_rect: egui::Rect) -> egui::Response {
//     let rect_center = available_rect.center();
//     let color = app.color_data.widget_color;
//     let detail_color = app.color_data.widget_detail_color;
    
//     let h_offset = vec2(2.5 * SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
//     let rect = egui::Rect::from_min_max(
//         rect_center + h_offset - vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y),
//         rect_center + h_offset + vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y));

//     // println!("space: {}", (available_rect.width()-rect.right_center().x));

//     let mut response = ui.allocate_rect(rect, egui::Sense::click());

//     // noting if it's been clicked
//     if response.clicked() {
//         if matches!(app.mode, QueueState::Loop) {
//             app.mode = QueueState::Next;
//         } else {
//             app.mode = QueueState::Loop;
//         }
//         response.mark_changed();
//     }

//     //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

//     //paint it on the screen
//     if ui.is_rect_visible(rect) {

//         let mut visuals = ui.style().interact_selectable(&response, true);
//         visuals.fg_stroke.width = 1.5;

//         let rect = rect.expand(visuals.expansion);
//         let radius = 0.5 * rect.height();
//         let center = egui::pos2(rect.center().x, rect.center().y);

//         ui.painter().rect(rect, radius, color, visuals.fg_stroke);
//         ui.painter().text(
//             center, 
//             egui::Align2::CENTER_CENTER, "🔁", 
//             FontId::proportional(rect.height()-4.0), 
//             detail_color);
//     }
//     response
// }


// pub fn button_decals(mode: &mut QueueState, ui: &mut egui::Ui, available_rect: egui::Rect) {
//     //let rect_center = available_rect.center();
    
//     let h_offset = vec2(2.5 * SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
//     let v_offset = vec2(0.0, 30.0);

//     let shuff_center = (available_rect.center().to_vec2()-h_offset+v_offset).to_pos2();
//     let loop_center = (available_rect.center().to_vec2()+h_offset+v_offset).to_pos2();
    
//     match mode {
//         QueueState::Shuffle => ui.painter().circle_filled(shuff_center, 5.0, Color32::WHITE),
//         QueueState::Loop => ui.painter().circle_filled(loop_center, 5.0, Color32::WHITE),
//         _ => (),
//     }
// }

// pub fn volume_slider( ui: &mut egui::Ui, app: &mut SpogApp, min: f32, max: f32) {
//     let available_rect = ui.max_rect();

//     // Pulled from the Loop Button to figure out where we need to stop the slider if there's not enough space for full size
//     let h_offset = vec2(2.5 * SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
//     let loop_x = available_rect.center().x + h_offset.x + ui.spacing().interact_size.y;

//     let (bot_right, top_left) = if available_rect.width()<979.0 {
//         (
//             available_rect.right_center() - vec2(20.0, -10.0),
//             available_rect.right_center() - vec2(20.0, -10.0) - vec2(available_rect.width()-loop_x, 20.0) + vec2(30.0, 0.0)
//         )
//     } else if available_rect.width()<1199.0 {
//         (
//             available_rect.right_center() - vec2(20.0+55.0*((available_rect.width()-979.0)/(1199.0-979.0)), -10.0),
//             available_rect.right_center() - vec2(20.0+55.0*((available_rect.width()-979.0)/(1199.0-979.0)), -10.0) - vec2(300.0, 20.0)
//         )
//     } else {
//         (
//             available_rect.right_center() - vec2(75.0, -10.0),
//             available_rect.right_center() - vec2(75.0, -10.0)- vec2(300.0, 20.0)
//         )
//     };

//     let bar = egui::Rect::from_two_pos(top_left, bot_right);
//     let offset = top_left.x;
//     let v_offset = bar.center().y;
//     let slide_len = bar.width();

//     let center = pos2(val_to_pos(app.volume, offset, min, max, slide_len) as f32, v_offset);
//     let knob = egui::Rect::from_center_size(center, vec2(24.0,24.0));

//     let knob_response = ui.allocate_rect(knob, egui::Sense::click_and_drag());
//     let bar_response = ui.allocate_rect(bar.shrink2(vec2(12.0, 0.0)), egui::Sense {click: true, drag: true, focusable: true});
//     if bar_response.clicked() {
//         let pos = bar_response.interact_pointer_pos().unwrap().x;
//         app.volume = pos_to_val(pos, offset, min, max, slide_len);
//         app.sink.set_volume(app.volume);
//     }

//     if bar_response.hovered() {
//         let delta = ui.ctx().input().scroll_delta.y;
//         let new_pos = val_to_pos(app.volume, offset, min, max, slide_len) + delta*0.2;
//         app.volume = pos_to_val(new_pos, offset, min, max, slide_len).clamp(min, max);
//         if delta != 0.0 {
//             app.sink.set_volume(app.volume);
//         } 
//     }

//     if knob_response.dragged() {
//         let delta = knob_response.drag_delta().x;
//         let new_pos = val_to_pos(app.volume, offset, min, max, slide_len) + delta;
//         app.volume = pos_to_val(new_pos, offset, min, max, slide_len).clamp(min, max);
//         app.sink.set_volume(app.volume);
//     }

//     ui.painter().rect(bar, egui::Rounding::same(10.0), SLIDER_BACKGROUND, egui::Stroke {width: 0.0, color: BLACK});
//     if available_rect.width()>1300.0 {
//         let text = if !app.settings.muted {
//             String::from("🔊")
//         } else if !app.settings.muted && app.volume<0.075 {
//             String::from("🔉")
//         } else {
//             String::from("🔈")
//         };
//         let speaker_rect = ui.painter().text(
//             bar.left_center()-vec2(20.0, 0.0), 
//             egui::Align2::CENTER_CENTER, 
//             text, FontId::proportional(32.0), 
//             LIGHT_GREY);
//         let speaker_response = ui.allocate_rect(speaker_rect, egui::Sense::click());
//         if speaker_response.hovered() {
//             let mask = egui::Color32::from_white_alpha(11);
//             ui.painter().rect_filled(speaker_rect, egui::Rounding::none(), mask);
//         }
//         if speaker_response.clicked() {
//             app.settings.muted = !app.settings.muted;
//             if app.settings.muted {
//                 app.sink.set_volume(0.0)
//             } else {
//                 app.sink.set_volume(app.volume)
//             }
//         }
//     }

//     if bar_response.hovered() | knob_response.is_pointer_button_down_on() {
//         ui.painter().circle(center, 12.0, app.color_data.widget_color, egui::Stroke {width: 3.0, color: WHITE});
//     } else {
//         ui.painter().circle_filled(center, 12.0, app.color_data.widget_color);
//     }
// }

// fn pos_to_val(pos: f32, offset: f32, min: f32, max: f32, slide_len: f32) -> f32 {
//     min + ((pos-(offset+12.0))/(slide_len-24.0))*(max - min)
// }

// fn val_to_pos(val: f32, offset: f32, min: f32, max: f32, slide_len: f32) -> f32 {
//     (offset+12.0) + ((slide_len-24.0)*(val-min)/(max-min))
// }

// pub fn width_from_str(text: &String, fonts: &egui::text::Fonts) -> f32 {
//     let mut width = 0.0;
//     for char in text.as_str().chars() {
//         width += egui::text::Fonts::glyph_width(
//             fonts,
//             &egui::FontId::proportional(28.0), 
//             char
//         );
//     }
//     width
// }

// pub fn show_current_song(ui: &mut egui::Ui, app: &mut SpogApp) {
//     let offset = ui.max_rect().left_center();
//     let width = ui.max_rect().width()*0.5 - (185.0);
//     let top_left = offset + vec2(20.0, -30.0);
//     let bot_right = top_left + vec2(width, 60.0);
//     let rect = egui::Rect::from_two_pos(top_left, bot_right);
//     ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
//     if app.track_list.len()>0 {
//         let song: &Song = &app.track_list[app.current_index];
//         let complete_song = if !song.title.is_empty() && !song.artist.is_empty() && !song.album.is_empty() {
//             true
//         } else {false};

//         use egui::Align;

//         let top_pos = rect.left_center() + vec2(0.0, -14.0);
//         if complete_song {
//             let bot_pos = rect.left_center() + vec2(0.0, 16.0);
//             let (title_gal, title_height) = singleline_galley(
//                 ui, width, &song.title.clone(), LIGHT_GREY, 28.0, Align::Center
//             );
//             let (artist_gal, artist_height) = singleline_galley(
//                 ui, width, &song.artist.clone(), LIGHT_GREY, 20.0, Align::Center
//             );
//             ui.painter().galley(top_pos - vec2(-0.5*width, 0.5*title_height), title_gal);
//             ui.painter().galley(bot_pos - vec2(-0.5*width, 0.5*artist_height), artist_gal);
//         } else {
//             let (path_gal, path_height) = singleline_galley(
//                 ui, width, &song.path.clone(), LIGHT_GREY, 28.0, Align::Center
//             );
//             ui.painter().galley(top_pos - vec2(-0.5*width, 0.5*path_height), path_gal);
//         }
//     }
// }

// pub fn settings_button(ui: &mut egui::Ui, app: &mut SpogApp) {
//     let available_rect = ui.max_rect();
//     let center = available_rect.right_center() - vec2(30.0, 0.0);
//     let rect = egui::Rect::from_center_size(center, vec2(40.0,40.0));
//     let cog_response = ui.allocate_rect(rect, egui::Sense::click());

//     if cog_response.clicked() {
//         println!("woohoo you clicked the button!");
//         app.window_bools.settings = !app.window_bools.settings;
//     }

//     if cog_response.hovered() {
//         ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 2.0, color:WHITE});
//         ui.painter().text(
//             center, 
//             egui::Align2::CENTER_CENTER, "⚙", 
//             FontId::proportional(30.0), 
//             epaint::Color32::WHITE);
//     } else {
//         ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 1.0, color:LIGHT_GREY});
//         ui.painter().text(
//             center, 
//             egui::Align2::CENTER_CENTER, "⚙", 
//             FontId::proportional(30.0), 
//             LIGHT_GREY);
//     } 
//     // let button = egui::Button::new("⚙");     
// }

pub fn settings_window(app: &mut SpogApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::Window::new("Settings").open(&mut app.window_bools.settings).show(ctx, |ui| {
        ui.label(&app.direct_path);
        ui.horizontal(|ui| {
            ui.label("New Directory:");
            ui.text_edit_singleline(&mut app.settings.direct_buf);
            let response = ui.button("Confirm new directory path?");
            let popup_id = ui.make_persistent_id("directory_error");
            if response.clicked() {
                if Path::exists(Path::new(&app.settings.direct_buf)) {
                    app.direct_path = app.settings.direct_buf.clone();
                    app.current_index = 0;
                    let track_paths = get_tracks(&app.direct_path);
                    let mut songs = vec![];
                    for i in 0..track_paths.len() {
                        if let Some(t) = get_song(app.direct_path.as_str(), track_paths[i].as_str(), i) {
                            songs.push(t)
                        }
                    }
                    app.track_list = songs;
                    //app.boolean = false;
                } else {
                    ui.memory().open_popup(popup_id);
                }
            }
            egui::popup_below_widget(ui, popup_id, &response, |ui| {
                ui.label("Couldn't find that directory!");
            });
        });
        
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

        if ui.button("Switch App Mode").clicked() {
            app.settings.mini_mode = !app.settings.mini_mode;
            // frame.set_visible(!app.settings.mini_mode);
        }
    });
}

pub fn shuffle_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    let color = app.color_data.widget_color;
    let detail_color = app.color_data.widget_detail_color;

    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));

    let mut response = ui.allocate_rect(rect, egui::Sense::click());
    
    // noting if it's been clicked
    if response.clicked() {
        if matches!(app.mode, QueueState::Shuffle) {
            app.mode = QueueState::Next;
        } else {
            app.mode = QueueState::Shuffle;
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
    if response.clicked() {
        app.go_back();
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
    // println!("{}", rect.center().x);
    // making space for it
    let mut response = ui.allocate_rect(rect, egui::Sense::click());

    // noting if it's been clicked
    if response.clicked() {
        app.on = !app.on;
        response.mark_changed();
        if app.on {
            println!("Now playing music");
            app.sink.play();
        } else { 
            app.sink.pause();
            println!("Music paused")
        }
    }

    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, app.on, ""));

    //paint it on the screen
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
            //ui.painter().extend(pause);
            ui.painter().text(
                center, 
                egui::Align2::CENTER_CENTER, "⏸", 
                FontId::proportional(rect.height()-6.0), 
                detail_color);
        } else {
            //ui.painter().add(play); ▶
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

    // noting if it's been clicked
    if response.clicked() {
        app.skip_song();
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
            egui::Align2::CENTER_CENTER, "⏭", 
            FontId::proportional(rect.height()-4.0), 
            detail_color);
    }
    response
}

pub fn loop_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) -> egui::Response {
    let color = app.color_data.widget_color;
    let detail_color = app.color_data.widget_detail_color;

    // println!("space: {}", (available_rect.width()-rect.right_center().x));

    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));

    let mut response = ui.allocate_rect(rect, egui::Sense::click());

    // noting if it's been clicked
    if response.clicked() {
        if matches!(app.mode, QueueState::Loop) {
            app.mode = QueueState::Next;
        } else {
            app.mode = QueueState::Loop;
        }
        response.mark_changed();
    }

    //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    //paint it on the screen
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

pub fn button_decals(mode: &mut QueueState, ui: &mut egui::Ui, shuffle_nw: egui::Pos2, loop_nw: egui::Pos2) {
    let center = match mode {
        QueueState::Shuffle => {
            shuffle_nw + vec2(0.5*BUTTON_SIDE, BUTTON_SIDE+10.0)        
        },
        QueueState::Loop => {
            loop_nw + vec2(0.5*BUTTON_SIDE, BUTTON_SIDE+10.0)
            
        },
        _ => egui::Pos2::ZERO,
    };
    if let QueueState::Shuffle | QueueState::Loop = mode {
        ui.painter().circle_filled(center, 5.0, Color32::WHITE);
    }
}

pub fn volume_slider( ui: &mut egui::Ui, app: &mut SpogApp, bar: egui::Rect, min: f32, max: f32) {
    // let available_rect = ui.max_rect();

    // Pulled from the Loop Button to figure out where we need to stop the slider if there's not enough space for full size
    // let h_offset = vec2(2.5 * SPACING, 0.0) + vec2(2.0 * ui.spacing().interact_size.y, 0.0);
    // let loop_x = available_rect.center().x + h_offset.x + ui.spacing().interact_size.y;

    // let available_w = available_rect.left();
    // let h_offset = 5.0*(BUTTON_SIDE+SPACING);
    // let loop_x = available_w + h_offset + BUTTON_SIDE;

    // let (bot_right, top_left) = (
    //     available_rect.left_top() + vec2(loop_x+SPACING, 0.5*BUTTON_SIDE+SPACING+SPACING),
    //     available_rect.right_top() + vec2(-SPACING, SPACING+BUTTON_SIDE+SPACING)
    // );

    // let bar = egui::Rect::from_two_pos(top_left, bot_right);
    // let offset = top_left.x;
    // let v_offset = bar.center().y;
    // let slide_len = bar.width();

    let center = pos2(val_to_pos(app.volume, bar.left(), min, max, bar.width()) as f32, bar.center().y);
    let knob = egui::Rect::from_center_size(center, vec2(24.0,24.0));

    let knob_response = ui.allocate_rect(knob, egui::Sense::click_and_drag());
    let bar_response = ui.allocate_rect(bar.shrink2(vec2(12.0, 0.0)), egui::Sense {click: true, drag: true, focusable: true});
    if bar_response.clicked() {
        let pos = bar_response.interact_pointer_pos().unwrap().x;
        app.volume = pos_to_val(pos, bar.left(), min, max, bar.width());
        app.sink.set_volume(app.volume);
        println!("{}", exp_p_to_v(pos, bar.left(), bar.width()));
    }

    if bar_response.hovered() {
        let delta = ui.ctx().input().scroll_delta.y;
        let new_pos = val_to_pos(app.volume, bar.left(), min, max, bar.width()) + delta*0.2;
        app.volume = pos_to_val(new_pos, bar.left(), min, max, bar.width()).clamp(min, max);
        if delta != 0.0 {
            app.sink.set_volume(app.volume);
        } 
    }

    if knob_response.dragged() {
        let delta = knob_response.drag_delta().x;
        let new_pos = val_to_pos(app.volume, bar.left(), min, max, bar.width()) + delta;
        app.volume = pos_to_val(new_pos, bar.left(), min, max, bar.width()).clamp(min, max);
        app.sink.set_volume(app.volume);
    }

    ui.painter().rect(bar, egui::Rounding::same(10.0), SLIDER_BACKGROUND, egui::Stroke {width: 0.0, color: BLACK});
    
    let text = if !app.settings.muted {
        String::from("🔊")
    } else if !app.settings.muted && app.volume<0.075 {
        String::from("🔉")
    } else {
        String::from("🔈")
    };
    let speaker_rect = ui.painter().text(
        bar.left_center()-vec2(20.0, 0.0), 
        egui::Align2::CENTER_CENTER, 
        text, FontId::proportional(32.0), 
        LIGHT_GREY);
    let speaker_response = ui.allocate_rect(speaker_rect, egui::Sense::click());
    if speaker_response.hovered() {
        let mask = egui::Color32::from_white_alpha(11);
        ui.painter().rect_filled(speaker_rect, egui::Rounding::none(), mask);
    }
    if speaker_response.clicked() {
        app.settings.muted = !app.settings.muted;
        if app.settings.muted {
            app.sink.set_volume(0.0)
        } else {
            app.sink.set_volume(app.volume)
        }
    }
    if ui.max_rect().width()>1045.0 {
        
    }

    if bar_response.hovered() | knob_response.is_pointer_button_down_on() {
        ui.painter().circle(center, 12.0, app.color_data.widget_color, egui::Stroke {width: 3.0, color: WHITE});
    } else {
        ui.painter().circle_filled(center, 12.0, app.color_data.widget_color);
    }
}

fn pos_to_val(pos: f32, offset: f32, min: f32, max: f32, slide_len: f32) -> f32 {
    min + ((pos-(offset+12.0))/(slide_len-24.0))*(max - min)
}

fn val_to_pos(val: f32, offset: f32, min: f32, max: f32, slide_len: f32) -> f32 {
    (offset+12.0) + ((slide_len-24.0)*(val-min)/(max-min))
}

// UNFINISHED exponential volume_sliders
fn exp_p_to_v(pos: f32, offset: f32, slide_len: f32) -> f32 {
    let x = 100.0*(pos-offset)/slide_len;
    println!("{}",((pos.powi(2)-offset)/slide_len));
    (1.9_f32.powf((pos.powi(2)-offset)/slide_len))-1.9_f32.powf(offset/slide_len)
}

fn exp_v_to_p() -> f32 {
    0.0
}

// Width of text in search bar, 
pub fn width_from_str(text: &String, fonts: &egui::text::Fonts) -> f32 {
    let mut width = 0.0;
    for char in text.as_str().chars() {
        width += egui::text::Fonts::glyph_width(
            fonts,
            &egui::FontId::proportional(28.0), 
            char
        );
    }
    width
}

pub fn show_current_song(ui: &mut egui::Ui, app: &mut SpogApp, rect: egui::Rect) {
    // let offset = ui.max_rect().left_center();
    let width = rect.width();
    
    ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
    if app.track_list.len()>0 {
        let song: &Song = &app.track_list[app.current_index];
        let complete_song = if !song.title.is_empty() && !song.artist.is_empty() && !song.album.is_empty() {
            true
        } else {false};

        use egui::Align;

        if app.settings.mini_mode {
            let left_pos = rect.left_center();
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
                let (path_gal, path_height) = singleline_galley(
                    ui, width, &song.path.clone(), LIGHT_GREY, 28.0, Align::Center
                );
                ui.painter().galley(left_pos - vec2(-0.5*width, 0.5*path_height), path_gal);
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
                let (path_gal, path_height) = singleline_galley(
                    ui, width, &song.path.clone(), LIGHT_GREY, 28.0, Align::Center
                );
                ui.painter().galley(top_pos - vec2(-0.5*width, 0.5*path_height), path_gal);
            }
        }
    }
}

pub fn settings_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) {
    // let available_rect = ui.max_rect();
    // let center = available_rect.right_top() + vec2(-(20.0 + SPACING), SPACING+BUTTON_SIDE);
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
            epaint::Color32::WHITE);
    } else {
        ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 1.0, color:LIGHT_GREY});
        ui.painter().text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, "⚙", 
            FontId::proportional(30.0), 
            LIGHT_GREY);
    } 
    // let button = egui::Button::new("⚙");
        
}

// Toggles whether the app is in full or portable mode
pub fn mode_button(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: egui::Pos2) {
    let rect = egui::Rect::from_min_size(rect_nw, vec2(BUTTON_SIDE, BUTTON_SIDE));
    let response = ui.allocate_rect(rect, egui::Sense::click());
    
    if response.clicked() {
        app.settings.mini_mode = !app.settings.mini_mode;
    }
    
    if response.hovered() {
        ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 2.0, color:WHITE});
        ui.painter().text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, "⇲", 
            FontId::proportional(30.0), 
            epaint::Color32::WHITE);
    } else {
        ui.painter().circle(rect.center(), 20.0, SLIDER_BACKGROUND, egui::Stroke{width: 1.0, color:LIGHT_GREY});
        ui.painter().text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, "⇲", 
            FontId::proportional(30.0), 
            LIGHT_GREY);
    }
}
pub fn render_search_bar(ui: &mut egui::Ui, app: &mut SpogApp) {
    let top_left = ui.max_rect().left_top() + vec2(10.0, 5.0);
    let bot_right = top_left + vec2(400.0, 40.0);
    let bar = egui::Rect::from_two_pos(top_left, bot_right);
    let magnifying_pos = bar.left_center() + vec2(10.0, 0.0);

    let bar_response = ui.allocate_rect(bar, egui::Sense {click: true, drag: true, focusable: true});
    
    if bar_response.clicked() {
        app.filter_data.active = true;
        // test_width(ui);
    } if bar_response.clicked_elsewhere() {
        app.filter_data.active = false;
    } if app.filter_data.active {
        for event in &ui.ctx().input().events {
            if let egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers: _ } = event {
                app.filter_data.active = false;
            }
        }
    }

    let shift = if app.filter_data.text_width < 335.0 {
        0.0
    } else {
        app.filter_data.text_width-335.0
    };

    let offset = vec2(45.0-shift,0.0);

    let line = epaint::Shape::LineSegment {
        points: [
            bar.left_top()+vec2(app.filter_data.text_width, 5.0)+offset, 
            bar.left_top()+vec2(app.filter_data.text_width, 35.0)+offset
            ], 
        stroke: egui::Stroke{width: 2.0, color: BLACK}
    };

    if app.filter_data.active {
        for event in &ui.ctx().input().events {
            if let egui::Event::Text(t) = event {
                let width = width_from_str(t, &app.filter_data.fonts);
                app.filter_data.text_width += width;
                app.filter_data.field.push_str(t.clone().as_str());
            }
            else if let egui::Event::Key { key: egui::Key::Backspace, pressed: true, modifiers: _ } = event {
                if !app.filter_data.field.is_empty() {
                    let char = app.filter_data.field.pop().unwrap().to_string();
                    app.filter_data.text_width -= width_from_str(&char, &app.filter_data.fonts);
                }
            }
            // println!("{}", app.filter_data.text_width);
        }

        ui.painter().rect_filled(bar, egui::Rounding::same(20.0), WHITE);
        ui.painter().text(
            magnifying_pos, 
            egui::Align2::LEFT_CENTER, 
            "🔍", egui::FontId::proportional(30.0), 
            BLACK
        );
        ui.painter().add(line);
    } else {
        ui.painter().rect_filled(bar, egui::Rounding::same(20.0), SLIDER_BACKGROUND);
        ui.painter().text(
            magnifying_pos, 
            egui::Align2::LEFT_CENTER, 
            "🔍", egui::FontId::proportional(30.0), 
            BLACK
        );
    }

    let viewport = egui::Rect::from_two_pos(bar.left_top()+vec2(45.0, 0.0), bar.right_bottom()-vec2(20.0,0.0));

    if app.filter_data.field.is_empty() {
        ui.painter().with_clip_rect(viewport).text(
            viewport.left_center()-vec2(shift, 0.0), 
            egui::Align2::LEFT_CENTER, 
            "Search by song title, artist, or file address", 
            egui::FontId::proportional(20.0), 
            LIGHT_GREY
        );
    }
    if app.filter_data.active {
        ui.painter().with_clip_rect(viewport).text(
            viewport.left_center()-vec2(shift, 0.0), 
            egui::Align2::LEFT_CENTER, 
            app.filter_data.field.clone(), 
            egui::FontId::proportional(28.0), 
            BLACK
        );
    } else {
        ui.painter().with_clip_rect(viewport).text(
            viewport.left_center()-vec2(shift, 0.0), 
            egui::Align2::LEFT_CENTER, 
            app.filter_data.field.clone(), 
            egui::FontId::proportional(28.0), 
            LIGHT_GREY
        );
    }
}