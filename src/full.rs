use crate::spogify::{SpogApp, Column, Sorting};
use crate::controls::{settings_button,render_search_bar,show_current_song,shuffle_button,prev,play_pause,next,loop_button,
    volume_slider,button_decals, mode_button, progress_bar};
use crate::{LIGHT_GREY, SLIDER_BACKGROUND, WHITE, BLACK, FERN};
use egui::{vec2, Pos2};
use std::f32::consts::FRAC_PI_6;
const SPACING: f32 = 10.0;
const BUTTON_SIDE: f32 = 40.0;

// Lays out panels for the full display version of the app
pub fn full_layout(ctx: &egui::Context, app: &mut SpogApp, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("search_bar").min_height(50.0).max_height(100.0).show(ctx, |ui| {
        let settings_nw = ui.max_rect().right_top() - vec2(BUTTON_SIDE+SPACING, -5.0);
        let bigmode_nw = settings_nw - vec2(BUTTON_SIDE+SPACING, 0.0);
        let loading_nw = bigmode_nw - vec2(1.5*BUTTON_SIDE+SPACING, -5.0);
        loading_bar(ui, app, loading_nw);

        // let progress_nw = ui.max_rect().left_top() + vec2(480.0, 15.0);
        // progress_bar(ui, app, progress_nw);
        
        settings_button(ui, app, settings_nw);
        mode_button(ui, app, bigmode_nw, frame);
        render_search_bar(ui, app);
        ui.separator();
        legend_bar(ui, app);
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
            
            egui::Rect::from_min_size(volume_nw - vec2(0.0, 20.0), vec2(width, 20.0))
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
        volume_slider(ui, app, volume_rect, app.settings.volume_range.0, app.settings.volume_range.1);
        button_decals(&mut app.mode, ui, shuffle_nw, loop_nw);
        current_response
    });

    // Fill the rest of the space with list of files
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui,|ui| {
            // depression(ui);
            app.render_play_buttons(ui, current_response.inner);
        });
    });
}

fn loading_bar(ui: &mut egui::Ui, app: &mut SpogApp, rect_nw: Pos2) {
    if app.song_loading.active {
        // Paint rects, be sure to paint them from the bottom up, not top down(or anchored to the bottom, not top at least)
        let num = app.settings.ref_time.elapsed().unwrap().as_secs_f32();
        for i in 0..3 {
            let multiplier = (num-(i as f32+2.0)*FRAC_PI_6).sin()*(num-(i as f32+2.0)*FRAC_PI_6).sin();
            let rect_sw = rect_nw + vec2(20.0*(i as f32), 30.0);
            let rect_ne = rect_sw - vec2(-15.0, 10.0+20.0*multiplier);
            let rect = egui::Rect::from_two_pos(rect_sw, rect_ne);
            ui.painter().rect_filled(rect, egui::Rounding::same(1.0), FERN);
        }
  
        // ui.allocate_ui_at_rect(rect, |ui| {
        //     ui.image(app.images.circular_arrow.texture_id(ui.ctx()), ui.max_rect().size());
        // });
    }
}

// Shows "title", "artist", "album" above respective columns
pub fn legend_bar(ui: &mut egui::Ui, app: &mut SpogApp) {
    let (_, rect) = ui.allocate_space(vec2(ui.max_rect().width(), 30.0));
    let title_pos = rect.left_center() + vec2(65.0, 0.0);
    let artist_pos = title_pos + vec2(100.0, 0.0);
    let album_pos = rect.left_center() + vec2(0.5*ui.max_rect().width(), 0.0);
    let date_pos = rect.left_center() + vec2(0.83*ui.max_rect().width(), 0.0);

    let title = ui.painter().text(
        title_pos, 
        egui::Align2::LEFT_CENTER, 
        "Title", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
    let title_sort = egui::Rect::from_min_size(title.right_top()+vec2(10.0, 0.0), vec2(50.0,20.0));
    col_sort_button(title_sort, Column::Title, app, ui);
    
    let artist = ui.painter().text(
        artist_pos, 
        egui::Align2::LEFT_CENTER, 
        "Artist", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
    let artist_sort = egui::Rect::from_min_size(artist.right_top()+vec2(10.0, 0.0), vec2(50.0,20.0));
    col_sort_button(artist_sort, Column::Artist, app, ui);
    
    let album = ui.painter().text(
        album_pos, 
        egui::Align2::LEFT_CENTER, 
        "Album", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
    let album_sort = egui::Rect::from_min_size(album.right_top()+vec2(10.0, 0.0), vec2(50.0,20.0));
    col_sort_button(album_sort, Column::Album, app, ui);

    let date = ui.painter().text(
        date_pos, 
        egui::Align2::LEFT_CENTER, 
        "Date", 
        egui::FontId::proportional(28.0), 
        LIGHT_GREY
    );
    let date_sort = egui::Rect::from_min_size(date.right_top()+vec2(10.0, 0.0), vec2(50.0,20.0));
    col_sort_button(date_sort, Column::Date, app, ui);
}

// Make a button at provided rect that sorts by that column
fn col_sort_button(ref_rect: egui::Rect, col: Column, app: &mut SpogApp, ui: &mut egui::Ui) {
    let nw = ref_rect.left_top();
    // Rect sized based on legend bar "Title...etc." text size
    let rect = egui::Rect::from_min_size(nw, vec2(28.0, 28.0));
    // ui.painter().rect(rect, egui::Rounding::same(2.0), SLIDER_BACKGROUND, egui::Stroke{width: 1.0, color:LIGHT_GREY});
    let response = ui.allocate_rect(rect, egui::Sense::click());
    if response.clicked() && !app.song_loading.active {
        if col == app.settings.sorting.column {
            if app.settings.sorting.alphebetical {
                app.settings.sorting.alphebetical = false;
            } else {
                app.settings.sorting = Sorting {alphebetical: true, column: Column::FileName}
            }
        } else {
            app.settings.sorting = Sorting {alphebetical: true, column: col};

        }
        app.sort_tracklist();
    }

    let color = if response.hovered() {
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
        WHITE
    } else {
        LIGHT_GREY
    };
    
    let text = if col == app.settings.sorting.column {
        if app.settings.sorting.alphebetical {
            "▲".to_string()
        } else {
            "▼".to_string()
        }
    } else {
        "─".to_string()
    };

    ui.painter().text(
        rect.center(), 
        egui::Align2::CENTER_CENTER, 
        text, 
        egui::FontId::monospace(20.0), 
        color
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