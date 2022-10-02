use crate::spogify::SpogApp;
use crate::controls::{settings_button,render_search_bar,show_current_song,shuffle_button,prev,play_pause,next,loop_button,
    volume_slider,button_decals, mode_button};
use egui::vec2;

const SPACING: f32 = 10.0;
const BUTTON_SIDE: f32 = 40.0;

pub fn full_layout(ctx: &egui::Context, app: &mut SpogApp) {
    egui::TopBottomPanel::top("search_bar").min_height(50.0).max_height(100.0).show(ctx, |ui| {
        let settings_nw = ui.max_rect().right_center() - vec2(SPACING+BUTTON_SIDE, 20.0);
        let bigmode_nw = settings_nw - vec2(BUTTON_SIDE+SPACING, 0.0);
        
        settings_button(ui, app, settings_nw);
        mode_button(ui, app, bigmode_nw);
        render_search_bar(ui, app);
    });

    egui::TopBottomPanel::bottom("playback_controls").min_height(100.0).show(ctx, |ui| {
        let shuffle_nw = ui.max_rect().center() - vec2(3.0*BUTTON_SIDE+2.0*SPACING, 0.5*BUTTON_SIDE);
        let prev_nw = shuffle_nw + vec2(BUTTON_SIDE+SPACING, 0.0);
        let play_nw = prev_nw + vec2(BUTTON_SIDE+SPACING, -0.5*BUTTON_SIDE);
        let next_nw = play_nw + vec2(2.0*BUTTON_SIDE+SPACING, 0.5*BUTTON_SIDE);
        let loop_nw = next_nw + vec2(BUTTON_SIDE+SPACING, 0.0);
        // println!("{}", volume_nw.x-ui.max_rect().left());
        let volume_rect = {
            let (volume_nw, width) = if ui.available_width() < 1079.0 {
                let width = ui.max_rect().width() - (loop_nw.x + BUTTON_SIDE + SPACING + 30.0);
                (loop_nw + vec2(30.0 + BUTTON_SIDE+SPACING, 10.0), width)
            }  else {
                let width: f32 = 350.0;
                (ui.max_rect().right_center() - vec2(SPACING+width, 10.0), width)
            };
            // println!("{}", ui.available_width());
            
            egui::Rect::from_min_size(volume_nw, vec2(width, 20.0))
        };        
        
        let song_rect = {
            let width = ui.max_rect().width()*0.5 - (185.0);
            let top_left = ui.max_rect().left_center() + vec2(20.0, -30.0);
            let bot_right = top_left + vec2(width, 60.0);
            egui::Rect::from_two_pos(top_left, bot_right)
        };

        // Buttons defined in controls.rs
        show_current_song(ui, app, song_rect);
        shuffle_button(ui, app, shuffle_nw);
        prev(ui, app, prev_nw);
        play_pause(ui, app, play_nw);
        next(ui, app, next_nw);
        loop_button(ui, app, loop_nw);
        volume_slider(ui, app, volume_rect, 0.0, 0.3);
        button_decals(&mut app.mode, ui, shuffle_nw, loop_nw);
        // println!("{}", ui.max_rect().width());
    });

    // Fill the rest of the space with list of files
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui,|ui| {
            app.render_play_buttons(ui);
        });
    });
}