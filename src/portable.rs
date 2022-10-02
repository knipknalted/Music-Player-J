use eframe::epaint::vec2;

use crate::spogify::SpogApp;
use crate ::controls::{show_current_song,shuffle_button,prev,play_pause,next,loop_button,volume_slider,settings_button,button_decals, mode_button};

const SPACING: f32 = 10.0;
const BUTTON_SIDE: f32 = 40.0;

pub fn portable_layout(ctx: &egui::Context, app: &mut SpogApp) {
    egui::CentralPanel::default().show(ctx, |ui| {

        let bigmode_nw = ui.max_rect().left_top() + vec2(40.0, SPACING+0.5*BUTTON_SIDE);
        // let shuffle_nw = ui.max_rect().left_top() + vec2(SPACING, SPACING+0.5*BUTTON_SIDE);
        let shuffle_nw = bigmode_nw + vec2(SPACING+BUTTON_SIDE + 62.0, 0.0);
        let prev_nw = shuffle_nw + vec2(BUTTON_SIDE+SPACING, 0.0);
        let play_nw = prev_nw + vec2(BUTTON_SIDE+SPACING, -0.5*BUTTON_SIDE);
        let next_nw = play_nw + vec2(2.0*BUTTON_SIDE+SPACING, 0.5*BUTTON_SIDE);
        let loop_nw = next_nw + vec2(BUTTON_SIDE+SPACING, 0.0);
        // let volume_nw = loop_nw + vec2(BUTTON_SIDE+SPACING+30.0, 10.0+BUTTON_SIDE);
        // println!("{}", loop_nw.x-shuffle_nw.x);
        let mut volume_rect = egui::Rect::EVERYTHING;
        volume_rect.set_top(play_nw.y+2.0*BUTTON_SIDE+20.0);
        volume_rect.set_left(ui.max_rect().left()+SPACING+30.0);
        volume_rect.set_right(ui.max_rect().right()-SPACING);
        volume_rect.set_height(20.0);
        // let volume_rect = egui::Rect::from_min_size(volume_nw, vec2(350.0, 20.0));
        // let bigmode_nw = loop_nw + vec2(BUTTON_SIDE+SPACING, 0.0);
        let settings_nw = loop_nw +vec2(BUTTON_SIDE+SPACING+62.0, 0.0);
        
        let song_rect = {
            let song_nw = ui.available_rect_before_wrap().right_bottom() + vec2(0.0, -40.0);
            let song_se = ui.available_rect_before_wrap().left_bottom();
            egui::Rect::from_two_pos(song_nw, song_se)
        };
         
        // Buttons defined in controls.rs
        show_current_song(ui, app, song_rect);
        shuffle_button(ui, app, shuffle_nw);
        prev(ui, app, prev_nw);
        play_pause(ui, app, play_nw);
        next(ui, app, next_nw);
        loop_button(ui, app, loop_nw);
        volume_slider(ui, app, volume_rect, 0.0, 0.3);
        mode_button(ui, app, bigmode_nw);
        settings_button(ui, app, settings_nw);
        button_decals(&mut app.mode, ui, shuffle_nw, loop_nw);
        // println!("{}", ui.max_rect().bottom());
    });
}