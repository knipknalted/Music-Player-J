use std::fs::{read_dir, DirEntry};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use crate::spogify::{SupportedFormat};
use crate::{LIGHT_GREY, SLIDER_BACKGROUND, FERN};
use egui::vec2;
use serde::{Serialize, Deserialize};

// Common functions for different song formats in refactoring
pub trait Songlike {}

// Information about a song and path to it
#[derive(Clone, PartialEq, Eq)]
pub struct Song {
    pub title: Rc<String>,    // name/title of the song
    pub artist: Rc<String>,  // artist name for the song
    pub album: Rc<String>,   // album name for the song
    pub file_name: Rc<String>, // file name
    pub format: SupportedFormat, // File type/extension for convenience, (unnecessary?)
    pub path: Rc<String>,    // file path to the song ***Maybe change to Path/PathBuf itself?
    pub date: OffsetDateTime,
    pub index: usize,    // number to keep track of what the index in the list is for the song, for convenience
    pub duration: Option<Duration>,
}

impl Song {
    pub fn is_complete(&self) -> bool {
        self.title.is_empty() && self.artist.is_empty() && self.album.is_empty() && self.date != OffsetDateTime::UNIX_EPOCH
    }

    pub fn to_cached_song(&self) -> CachedSong {
        CachedSong {
            title: (*self.title).clone(), 
            artist: (*self.artist).clone(), 
            album: (*self.album).clone(), 
        }
    }

    // Render a song in main window song list
    pub fn render_card(&self, ui: &mut egui::Ui, rect: egui::Rect, track_i: usize, layout_j: usize, current_index: usize, selected: Option<usize>) -> egui::Response {
        // let complete_song = !self.title.is_empty() && !self.artist.is_empty() && !self.album.is_empty();
        let card_response = ui.allocate_rect(rect, egui::Sense {click: true, drag: false, focusable: true});
        let digits: f32 = (track_i + 1).to_string().chars().count() as f32;

        // Position for the songs number in the list
        let num_pos = rect.left_center() + vec2(20.0, 0.0);

        let color = if track_i == current_index {
            FERN
        } else {
            LIGHT_GREY
        };

        if card_response.hovered() || {selected.is_some() && selected.unwrap() == layout_j} {
            ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
            ui.painter().text(
                num_pos, 
                egui::Align2::LEFT_CENTER, 
                "▶".to_string(), 
                egui::FontId::proportional(28.0), 
                color
            );
        } else {
            ui.painter().text(
                num_pos, 
                egui::Align2::LEFT_CENTER, 
                (track_i+1).to_string(), 
                egui::FontId::proportional(28.0), 
                color
            );
        }

        let first_pos = if self.title.is_empty() {
            rect.left_center() + vec2(20.0 + 35.0 + digits*10.0, 0.0)
        } else {
            rect.left_center() + vec2(20.0 + 35.0 + digits*10.0, -8.0)
        };

        // let first_pos = rect.left_center() + vec2(20.0 + 35.0 + digits*10.0, 0.0); 
        let total_width = rect.width() - first_pos.x - 3.0*20.0;

        use egui::Align;
        // If there's any metadata, use an actual label, otherwise just show the path
        // I think this can be refactored

        // let (main_gal, main_height) = if !self.title.is_empty() {
        //     singleline_galley(
        //         ui, 0.5*total_width, &self.title, LIGHT_GREY, 24.0, Align::LEFT
        //     )
        // } else {
        //     singleline_galley(
        //         ui, total_width, &self.file_name, LIGHT_GREY, 24.0, Align::LEFT
        //     )
        // };

        // Handling the title&artist or file name rendering
        if !self.title.is_empty() {
            // Positions for title, artist, and album text
            let (title_gal, title_height) = singleline_galley(
                ui, 0.5*total_width, &self.title, color, 24.0, Align::LEFT
            );
            let (artist_gal, artist_height) = singleline_galley(
                ui, 0.5*total_width, &self.artist, color, 18.0, Align::LEFT
            );

            let artist_pos = first_pos + vec2(0.0, 18.0);

            ui.painter().galley(first_pos - vec2(0.0, 0.5*title_height), title_gal);
            ui.painter().galley(artist_pos - vec2(0.0, 0.5*artist_height), artist_gal);
        } else {
            let file_name = &self.file_name;
            let (name_gal, name_height) = singleline_galley(
                ui, total_width, file_name, color, 24.0, Align::LEFT
            );
            ui.painter().galley(first_pos - vec2(0.0, 0.5*name_height), name_gal);
        }

        if !self.title.is_empty() && !self.album.is_empty() {
            let (album_gal, album_height) = singleline_galley(
                ui, 0.25*total_width, &self.album, color, 20.0, Align::LEFT
            );
            let album_pos = first_pos + vec2(0.5*total_width + 20.0, 8.0);
            ui.painter().galley(album_pos- vec2(0.0, 0.5*album_height), album_gal);
        }

        if self.date != OffsetDateTime::UNIX_EPOCH {
            let text = Rc::new(format!("{} {}, {}", self.date.month(), self.date.day(), self.date.year()));
            let (date_gal, date_height) = singleline_galley(
                ui, 0.25*total_width, &text, color, 20.0, Align::LEFT
            );
            let date_pos = first_pos + vec2(0.85*total_width + 20.0, 8.0);
            ui.painter().galley(date_pos- vec2(0.0, 0.5*date_height), date_gal);
        }

        card_response
    }
}

// Initialise and return vec of songs in directory (initialise is not async, so it doesn't read metadata and should be fast)
// Current slowdown is checking to make sure that a file is a valid audio file format
pub fn init_dir(dir_path: &str) -> Vec<Song> {
    let file_names = get_names_in_dir(dir_path);
    let mut track_list: Vec<Song> = vec![];
    
    for i in 0..file_names.len() {
        let try_format = string_ext(&file_names[i]);
        if let Some(format) = try_format {
            let file_path = format!(r"{}/{}", dir_path, file_names[i]);
            track_list.push(init_song(&file_path, &file_names[i], format, i));
        }
    }
    track_list
}

// Used on app launch/setting directory, does not read metadata to avoid large load time
pub fn init_song(file_path: &str, file_name: &str, format: SupportedFormat, index: usize) -> Song {
    Song {
        title: Rc::new("".to_string()),
        artist: Rc::new("".to_string()),
        album: Rc::new("".to_string()),
        file_name: Rc::new(file_name.to_string()),
        format: format,
        path: Rc::new(file_path.to_string()),
        date: OffsetDateTime::UNIX_EPOCH,
        index,
        duration: None,
    }
}

pub fn get_names_in_dir(dir_path: &str) -> Vec<String> {
    let mut file_vec: Vec<DirEntry> = vec![];
    
    // First we get each file in the file directory
    read_dir(dir_path).unwrap().for_each(|file| {
        // println!("{}", file_vec.capacity());
        match file {
            Ok(n) => file_vec.push(n),
            Err(e) => println!("oh fuck oh shit this broke it {}", e),
        }
    });

    let mut file_paths: Vec<String> = Vec::with_capacity(file_vec.len());

    // Then we get the name of each file and format it into the file's path
    file_vec.into_iter().for_each(|file| {
        file_paths.push(file.file_name().into_string().unwrap())
        
    });
    return file_paths;
}

pub fn string_ext(file_name: &str) -> Option<SupportedFormat> {
    let chars = file_name.chars();
    let mut buf: String = "".to_string();
    chars.for_each(|c| {
        if c != '.' {
            buf.push(c);
        } else  {
            buf.clear();
        }
    });
    match buf.as_str() {
        "mp3" => Some(SupportedFormat::mp3),
        "m4a" => Some(SupportedFormat::m4a),
        "flac" => Some(SupportedFormat::flac),
        "aac" => Some(SupportedFormat::aac),
        "ogg" => Some(SupportedFormat::ogg),
        "opus" => Some(SupportedFormat::ogg),
        _ => None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CachedSong {
    pub title: String,
    pub artist: String,
    pub album: String,
}

// Creates a galley that cuts off with ... if it exceeds the given size
pub fn singleline_galley(ui: &mut egui::Ui, max_width: f32, text: &Rc<String>, color: egui::Color32, size: f32, halign: egui::Align) -> (Arc<egui::Galley>, f32) {
    let job = egui::text::LayoutJob {
        sections: vec![egui::epaint::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: egui::TextFormat::simple(egui::FontId::proportional(size), color),
        }],
        text: text.to_string(),
        wrap: egui::epaint::text::TextWrapping {
            max_width, max_rows: 1, ..Default::default()
        },
        break_on_newline: false,
        halign,
        ..Default::default()
    };
    let height = job.font_height(&ui.fonts());
    (ui.fonts().layout_job(job), height)
}
