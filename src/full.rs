use crate::spogify::SpogApp;
use crate::controls::{settings_button,render_search_bar,show_current_song,shuffle_button,prev,play_pause,next,loop_button,
    volume_slider,button_decals, mode_button, LIGHT_GREY};
use egui::vec2;

const SPACING: f32 = 10.0;
const BUTTON_SIDE: f32 = 40.0;

// Lays out panels for the full display version of the app
pub fn full_layout(ctx: &egui::Context, app: &mut SpogApp) {
    egui::TopBottomPanel::top("search_bar").min_height(50.0).max_height(100.0).show(ctx, |ui| {
        let settings_nw = ui.max_rect().right_top() + vec2(-10.0-40.0, 5.0);
        let bigmode_nw = settings_nw - vec2(BUTTON_SIDE+SPACING, 0.0);
        
        settings_button(ui, app, settings_nw);
        mode_button(ui, app, bigmode_nw);
        render_search_bar(ui, app);
        ui.separator();
        legend_bar(ui);
    });

    let current_response = egui::TopBottomPanel::bottom("playback_controls").min_height(100.0).show(ctx, |ui| {
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
        let current_response = show_current_song(ui, app, song_rect);
        shuffle_button(ui, app, shuffle_nw);
        prev(ui, app, prev_nw);
        play_pause(ui, app, play_nw);
        next(ui, app, next_nw);
        loop_button(ui, app, loop_nw);
        volume_slider(ui, app, volume_rect, 0.0, 0.3);
        button_decals(&mut app.mode, ui, shuffle_nw, loop_nw);
        // println!("{}", ui.max_rect().width());
        current_response
    });

    // Fill the rest of the space with list of files
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui,|ui| {
            // legend_bar(ui);
            // depression(ui);
            app.render_play_buttons(ui, current_response.inner);
        });
    });
}

// Shows "title", "artist", "album" above respective columns
pub fn legend_bar(ui: &mut egui::Ui) {
    let (_, rect) = ui.allocate_space(vec2(ui.max_rect().width(), 30.0));
    let title_pos = rect.left_center() + vec2(65.0, 0.0);
    let artist_pos = title_pos + vec2(0.5*ui.max_rect().width(), 0.0);
    let album_pos = artist_pos + vec2(0.25*ui.max_rect().width(), 0.0);
    ui.painter().text(
        title_pos, 
        egui::Align2::LEFT_CENTER, 
        "Title", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
    ui.painter().text(
        artist_pos, 
        egui::Align2::LEFT_CENTER, 
        "Artist", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
    ui.painter().text(
        album_pos, 
        egui::Align2::LEFT_CENTER, 
        "Album", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
}

// meme
pub fn depression(ui: &mut egui::Ui) {
    let (_, rect) = ui.allocate_space(vec2(ui.max_rect().width(), 60.0));
    if ui.is_rect_visible(rect) {
        let pos = rect.center();
        ui.painter().text(
            pos, 
            egui::Align2::CENTER_CENTER, 
            "Powered by *actual* depression", 
            egui::FontId::proportional(48.0), 
            LIGHT_GREY
        );
    }
}