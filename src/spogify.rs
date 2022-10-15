use std::{fs::{read_dir,DirEntry, File},thread::{self, JoinHandle}, 
    io::BufReader,sync::{Arc, mpsc::{Sender, Receiver}}, collections::VecDeque, time::{Duration, SystemTime}, 
    f64::consts::PI, rc::Rc, path::{PathBuf, Path}
};
use rodio::{Decoder, Source};
use rand::{self, Rng};
use egui::vec2;
use serde::{Serialize, Deserialize};
use thin_vec::{thin_vec, ThinVec};
use crate::{controls::{settings_window,LIGHT_GREY, SLIDER_BACKGROUND}, portable::portable_layout, full::full_layout};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum QueueMode {
    Shuffle, // The next song played should be randomly selected from the available songs
    Next, // The next song played should be the next song in the list
    Loop, // The next song played should be the same song
}

// List of valid formats in looking for files, might want to refactor
const FORMATS: [&'static str; 2] = ["mp3", "mp4"];

use QueueMode::{Shuffle,Next,Loop};

// Information about a song and path to it
#[derive(Clone)]
pub struct Song {
    pub title: Rc<String>,    // name/title of the song
    pub artist: Rc<String>,  // artist name for the song
    pub album: Rc<String>,   // album name for the song
    pub file_name: Rc<String>, // file name
    // pub format: String, // File type/extension for convenience, (unnecessary?)
    pub path: Rc<String>,    // file path to the song
    pub index: usize,    // number to keep track of what the index in the list is for the song, for convenience
}

impl Song {
    // Render a song in main window song list
    pub fn render_card(&mut self, ui: &mut egui::Ui, rect: egui::Rect, num: usize) -> egui::Response {
        let complete_song = !self.title.is_empty() && !self.artist.is_empty() && !self.album.is_empty();
        let card_response = ui.allocate_rect(rect, egui::Sense {click: true, drag: false, focusable: true});
        let digits: f32 = (num + 1).to_string().chars().count() as f32;
        if card_response.hovered() {
            ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
        }

        // Position for the songs number in the list
        let num_pos = rect.left_center() + vec2(20.0, 0.0);
        let first_pos = rect.left_center() + vec2(20.0 + 35.0 + digits*10.0, 0.0);
        
        let total_width = rect.width() - first_pos.x - 3.0*20.0;

        ui.painter().text(
            num_pos, 
            egui::Align2::LEFT_CENTER, 
            (num+1).to_string(), 
            egui::FontId::proportional(24.0), 
            LIGHT_GREY
        );

        // let star_pos = rect.right_center() + vec2(-40.0, 0.0);
        // let color = if favs.contains(&self.index) {
        //     egui::Color32::GOLD
        // } else {
        //     SLIDER_BACKGROUND
        // };
        // let star = ui.painter().text(
        //     star_pos, 
        //     egui::Align2::LEFT_CENTER, 
        //     "★", 
        //     egui::FontId::proportional(28.0), 
        //     color
        // );
        // let star_sense = ui.allocate_rect(star, egui::Sense::click());

        // if star_sense.clicked() {
        //     if !favs.contains(&self.index) {
        //         favs.push(self.index);
        //     } else {
        //         let index = favs.iter().position(|i| *i == self.index);
        //         if let Some(index) = index {
        //             favs.swap_remove(index);
        //         }
        //     }
        // }

        use egui::Align;
        // If there's any metadata, use an actual label, otherwise just show the path
        // I think this can be refactored
        if complete_song {
            // Positions for title, artist, and album text
            let (title_gal, title_height) = singleline_galley(
                ui, 0.5*total_width, &self.title, LIGHT_GREY, 24.0, Align::LEFT
            );
            let (artist_gal, artist_height) = singleline_galley(
                ui, 0.25*total_width, &self.artist, LIGHT_GREY, 20.0, Align::LEFT
            );
            let (album_gal, album_height) = singleline_galley(
                ui, 0.25*total_width, &self.album, LIGHT_GREY, 20.0, Align::LEFT
            );
            let artist_pos = first_pos + vec2(0.5*total_width + 20.0, 0.0);
            let album_pos = artist_pos + vec2(0.25*total_width + 20.0, 0.0);
            
            ui.painter().galley(first_pos - vec2(0.0, 0.5*title_height), title_gal);
            ui.painter().galley(artist_pos - vec2(0.0, 0.5*artist_height), artist_gal);
            ui.painter().galley(album_pos- vec2(0.0, 0.5*album_height), album_gal);
        } else {
            let file_name = &self.file_name;
            let (name_gal, name_height) = singleline_galley(
                ui, total_width, file_name, LIGHT_GREY, 24.0, Align::LEFT
            );
            ui.painter().galley(first_pos - vec2(0.0, 0.5*name_height), name_gal);
        }
        // if current_response.clicked() && num == app_index {
        //     card_response.scroll_to_me(Some(egui::Align::Center));
        // }
        card_response
    }
}

// Creates a galley that cuts off with ... if it exceeds the given size
pub fn singleline_galley(ui: &mut egui::Ui, max_width: f32, text: &Rc<String>, color: egui::Color32, size: f32, halign: egui::Align) -> (Arc<egui::Galley>, f32) {
    let job = egui::text::LayoutJob {
        sections: vec![epaint::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: egui::TextFormat::simple(egui::FontId::proportional(size), color),
        }],
        text: text.to_string(),
        wrap: epaint::text::TextWrapping {
            max_width, max_rows: 1, ..Default::default()
        },
        break_on_newline: false,
        halign,
        ..Default::default()
    };
    let height = job.font_height(&ui.fonts());
    (ui.fonts().layout_job(job), height)
}

pub struct WindowBools {
    pub directory: bool, // Is the directory window open ***NO LONGER NEEDED
    pub settings: bool, // Is the settings window open
}

pub struct RgbShiftData {
    pub now: SystemTime, // Reference time
    pub elapsed: Duration, // Elapsed time since self.now, maybe unnecessary
    pub widget_color: egui::Color32, // rgb color for widgets
    pub widget_detail_color: egui::Color32, // color for details ex. text inside a widget, for if I want to make it reactive
}

pub struct FilterData {
    pub field: String, // What is currently typed in the search bar
    pub active: bool, // Is the search bar active
    pub selected: Option<usize>, // If the filter is active and isn't empty, which song is selected by arrow keys
    pub text_width: f32, // How much wide is the text in the search bar taking up (so I know where to put the cursor)
    pub ran: bool, // Used to make sure we only remake self.fonts once, probably unnecessary, remove later
    pub fonts: egui::text::Fonts, // The font info we need to figure out and update self.text_width
}

impl FilterData {
    pub fn default() -> Self {
        let fonts = egui::text::Fonts::new(
            1.0, 
            40000, 
            egui::FontDefinitions::default()
        );
        
        FilterData {field: String::new(), active: false, selected: None, text_width: 0.0, ran: false, fonts}
    }
}

pub struct Settings {
    pub color_shift: bool, // Is fancy changing rgb colored buttons on
    pub muted: bool, // Is the sink muted
    pub direct_buf: String, // Buffer for directory changes
    pub mini_mode: bool, // Whether the app is in mini_mode or not
    pub volume_mode: bool, // Whether the volume slider is logarithmic or linear
    pub favorites: Vec<usize>, // List of favorited song indices
    pub saved_dirs: Vec<String>, // Saved directories for easy switching
}

pub struct SongLoad {
    pub request: bool, // Set to true if need to ask app to read metadata from other scopes
    pub active: bool, // Easy toggle on/off to reduce conditionals in use
    pub tx: Sender<(String, String, String)>, // Sender for title, artist, album
    pub rx: Receiver<(String, String, String)>, // Receiver for title, artist, album
    pub position: usize, // Tracks position in tracklist
}

pub struct SpogApp {
    pub dir_path: String, // the location of the folder being used
    pub track_list: Vec<Song>, //the list of songs being used
    pub on: bool, // Whether the stream is paused or not
    pub volume: f32, // Stream volume
    pub mode: QueueMode, // Shuffle, loop, etc.
    pub current_index: usize, // index of the current song, used to 
    pub past_songs: VecDeque<usize>, // store indices of past n songs
    pub queued_song: Option<usize>, // Song queued to play next
    pub stream: rodio::OutputStream, // stream because it broke when I took it out
    pub stream_handle: rodio::OutputStreamHandle, // handle to the stream for resetting the sink
    pub sink: Arc<rodio::Sink>, // Atomic reference to a sink, so it's thread safe
    pub threads: ThinVec<JoinHandle<()>>, // temporary place to hold handles to threads, so they can be explicitly dropped, otherwise threads are never closed
    pub window_bools: WindowBools, // bools for if windows are open
    pub color_data: RgbShiftData, // Info using elapsed time to make fancy changing colors might take it out
    pub filter_data: FilterData, // Info used to render and use custom search bar
    pub settings: Settings, // Bools for which features are turned on, like color shift
    pub play_dur: SystemTime, // Time set every time we call run_track, used when going to previous track
    pub song_loading: SongLoad, // sender and reciever used to read song metadata in separate thread
}

impl SpogApp {
    // create a new app with some default settings, given a folder path to start from
    pub fn new(cc: &eframe::CreationContext) -> SpogApp {
        let mem: SpogMem = read_mem();

        // Set the app to initialize with dark mode and other custom widget settings
        let visuals: egui::Visuals = egui::Visuals::dark();      
        cc.egui_ctx.set_style(egui::Style{visuals, spacing: egui::style::Spacing{slider_width: 200.0, ..Default::default()}, ..Default::default()});

        let direct_path = String::from(&mem.dir_path); // What directory path are we using
        let file_names = get_names_in_dir(&direct_path); // Get all file paths from that directory
        for path in &file_names {
            // println!("{path}");
        }
        
        // Make an empty list of songs
        let mut track_list:Vec<Song> = Vec::with_capacity(file_names.len());
        // Then for each file path, see if it's a valid song, and add it to the list of songs if it is
        for i in 0..file_names.len() {
            let file_path = format!(r"{}/{}", &direct_path, &file_names[i]);
            let file_type = infer::get_from_path(&file_path).expect("Error getting type").expect("Unknown type");
            
            if FORMATS.iter().any(|f| f == &file_type.extension()) {
                track_list.push(init_song(&file_path, &file_names[i], i))
            }
            // println!("{}", file_type.extension());

            // println!("{}", file_names[i]);
        }

        // The song index to start with
        let current_index = mem.current_index;

        // Make vec that can pop from the back and push to the front to keep track of previously played songs
        let past_songs: VecDeque<usize> = VecDeque::with_capacity(20);

        // One stream and handle that we will continue to refer to
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        // The sink on startup isn't actually used, a new one is made when we play the song, maybe try to optimize
        let sink = Arc::new(rodio::Sink::try_new(&stream_handle).unwrap());
        sink.set_volume(0.05);

        // Initial values for window bools
        let window_bools = WindowBools {directory: false, settings: false};
        // Empty vec to manually drop thread handles
        let threads = thin_vec![];

        // Initial info for rgb shift
        let color_data = RgbShiftData {
            now: SystemTime::now(), 
            elapsed: Duration::new(0, 0), 
            widget_color: egui::Color32::from_rgb(104, 185, 115),
            widget_detail_color: egui::Color32::WHITE,
        };

        let filter_data = FilterData::default();
        
        let settings = Settings {
            color_shift: false, 
            muted: false, 
            direct_buf: String::new(), 
            mini_mode: false,
            volume_mode: mem.volume_mode,
            favorites: mem.favorites,
            saved_dirs: mem.saved_dirs,
        };
        let play_dur = SystemTime::now();
        let (tx, rx) = std::sync::mpsc::channel::<(String, String, String)>();
        let song_loading = SongLoad {request: false, active: false, tx, rx, position: 0,};

        let mut app = SpogApp {dir_path: direct_path, 
            track_list, 
            on: false, 
            volume: mem.volume, 
            mode: mem.queue_mode,
            current_index, 
            past_songs,
            queued_song: None,
            stream, 
            stream_handle, 
            sink, 
            threads,
            window_bools,
            color_data,
            filter_data,
            settings,
            play_dur,
            song_loading
        };
        // Starts the current song on the sink, otherwise the song on launch will be skipped(never given to the sink)
        app.run_track();
        app.sink.pause();
        app
    }

    // change our current song to be what our next one should be, but doesn't run it by itself
    pub fn fetch_next_song(&mut self) {
        // Put the song we just played into our song history
        // Limit our song history to 20 to reduce memory usage
        if self.past_songs.len() == Some(20).unwrap() {
            self.past_songs.pop_back();
        }
        self.past_songs.insert(0, self.current_index);
        // Figure out what our next song should be
        if let Some(queued_index) = self.queued_song {
            self.current_index = queued_index;
            self.queued_song = None;
        } else {
            let next_index = queue_track(&self.mode, self.current_index, self.track_list.len());
            self.current_index = next_index;
        }
    }

    // showing a button for every song 
    pub fn render_play_buttons(&mut self, ui: &mut egui::Ui, current_response: egui::Response) {
        const CARD_HEIGHT: f32 = 40.0;
        let offset_nw = ui.available_rect_before_wrap().left_top();
        let offset_ne = ui.available_rect_before_wrap().right_top();
        let mut j: f32 = 0.0;
        for i in 0..self.track_list.len() {
            let title = &self.track_list[i].title;
            let artist = &self.track_list[i].artist;
            let name = &self.track_list[i].file_name;

            // BIG LOGICAL MUMBO BREAKDOWN
            // '||' means OR, but it doesn't keep going if it's already true, '|' means regular OR
            // First, if the filter is empty, skip the other conditions and just show the card
            // If the filter isn't empty, then check for partial matches with the song title, artist, or album
            // Also check if the filter exactly matches the title (I don't think I need this, will come back to it)
            if self.filter_data.field.is_empty() || title.to_ascii_uppercase().contains(&self.filter_data.field.to_ascii_uppercase()) |
                artist.to_ascii_uppercase().contains(&self.filter_data.field.to_ascii_uppercase()) |
                name.to_ascii_uppercase().contains(&self.filter_data.field.to_ascii_uppercase()) || *self.filter_data.field == **title 
            {
                let card_nw = offset_nw + vec2(10.0, (j as f32)*(CARD_HEIGHT+10.0));
                let card_se = offset_ne + vec2(-10.0, CARD_HEIGHT + (j as f32)*(CARD_HEIGHT+10.0));
                let rect = egui::Rect::from_two_pos(card_nw, card_se);
                j += 1.0;
                ui.allocate_ui_at_rect(rect, |ui| {
                    if ui.is_rect_visible(rect) {
                        let response = self.track_list[i].render_card(ui, rect, i); 
                        if response.clicked() {
                            self.threads.clear();
                            self.sink.stop();
                            self.set_sink();
                            self.on = true;
        
                            // Put the previous song into history before playing the one clicked on
                            if self.past_songs.len() == Some(20).unwrap() {
                                self.past_songs.pop_back();
                            }
                            self.past_songs.insert(0, self.current_index);
                            // set the current index to be that of the song clicked on
                            self.current_index = i;
                            // let sink = self.sink.clone();
                            // run_track(sink, &self.track_list[self.current_index].path, &mut self.threads);
                            self.run_track();
                        }
                        response.context_menu(|ui| {
                            if ui.button("Queue Song").clicked() {
                                self.queued_song = Some(i);
                            }
                            if ui.button("favorite").clicked() {
                                if self.settings.favorites.contains(&i) {
                                    let index = self.settings.favorites.iter().position(|index| *index == i).unwrap();
                                    self.settings.favorites.swap_remove(index);
                                } else {
                                    self.settings.favorites.push(i);
                                }
                                // self.settings.favorites.push(i);
                            }
                        });
                    }
                    if current_response.clicked() && i == self.current_index {
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                    }
                });
            }
        }
    }

    // Take a sink, track_path, and thread vec (to drop the thread handle when we're done) and play the song 
    pub fn run_track(&mut self) {
        println!("{}", self.track_list[self.current_index].file_name);
        self.play_dur = SystemTime::now();
        
        let file = File::open(&*self.track_list[self.current_index].path);
        let source = if let Ok(file) = file {
            let file = BufReader::new(file);
            let source = Decoder::new(
                file);
            match source {
                Ok(t) => Some(t.buffered()),
                Err(_) => None,
            }
        } else {
            None
        };
    
        if let Some(source) = source {
            thread::scope(|s| {
                let _song_thread = s.spawn(|| {
                    self.sink.append(source);
                });
                // self.threads.push(song_thread);
            });
        }
    }

    pub fn set_sink(&mut self) {
        self.sink = Arc::new(rodio::Sink::try_new(&self.stream_handle).unwrap());
        self.sink.set_volume(self.volume);
    }

    pub fn skip_song(&mut self) {
        self.fetch_next_song();
        self.threads.clear();
        self.sink.stop();
        self.set_sink();
        // let sink = self.sink.clone();
        // run_track(sink, &self.track_list[self.current_index].path, &mut self.threads);
        self.run_track();
        self.on = true;
    }

    pub fn go_back(&mut self) {
        // Don't try to go back if there are no previous songs
        if self.past_songs.len() > 0 {
            // If the duration is less than 3 seconds, go to previous song
            if self.play_dur.elapsed().unwrap() > Duration::from_secs(3) {
                self.threads.clear();
                self.sink.stop();
                self.set_sink();
                // self.sink = set_sink(&self.stream_handle, self.volume);
                self.run_track();
            }
            // If the duration is greater than 3 seconds, restart the same song 
            else {
                self.current_index = self.past_songs.pop_front().unwrap();
            
                self.threads.clear();
                self.sink.stop();
                self.set_sink();
                // self.sink = set_sink(&self.stream_handle, self.volume);
                // let sink = self.sink.clone();
                // run_track(sink, &self.track_list[self.current_index].path, &mut self.threads);
                self.run_track();
            }
        }
    }

    // Updates the color for rgb color shift
    pub fn update_color(&mut self) {
        // Reset the duration at 10pi, which is one full cycle
        if self.color_data.elapsed > Duration::from_secs_f64(10.0*PI) {
            self.color_data.now = SystemTime::now();
            self.color_data.elapsed = Duration::new(0, 0);
        }
        self.color_data.elapsed = self.color_data.now.elapsed().unwrap();
        let angle = self.color_data.elapsed.as_secs_f64();

        let r = 255 - (((angle*0.1-(0.0*PI/4.0)).sin()*(angle*0.1-(0.0*PI/4.0)).sin())*128.5) as u8;
        let g = 255 - (((angle*0.1-(1.0*PI/3.0)).sin()*(angle*0.1-(1.0*PI/3.0)).sin())*128.5) as u8;
        let b = 255 - (((angle*0.1-(2.0*PI/3.0)).sin()*(angle*0.1-(2.0*PI/3.0)).sin())*128.5) as u8;

        let fill_color = egui::Color32::from_rgb(r, g, b);
        self.color_data.widget_color = fill_color;
    }

    // Synchronously: get valid files in directory and set track_list to it
    // Then Async: (try to) read metadata and pass it to app when ready
    pub fn refresh_directory(&mut self) {
        println!("called refresh directory");
        // self.dir_path has already been changed, this does not handle any directory changing itself
        let file_names = get_names_in_dir(&self.dir_path);

        // Get new track_list
        let new_songs = {
            let mut new_songs = vec![];
            for i in 0..file_names.len() {
                let file_path = format!(r"{}/{}", &self.dir_path, &file_names[i]);
                // File only gets added to list if it's a pre-approved format in FORMATS, may want to change/improve accuracy
                if let Ok(Some(file_type)) = infer::get_from_path(&file_path) {
                    if FORMATS.iter().any(|f| f == &file_type.extension()) {
                        new_songs.push(init_song(&file_path, &file_names[i], i))
                    }
                }
            }
            new_songs
        };
        println!("{} new songs", new_songs.len());
        
        self.track_list = new_songs;

        // Read song metadata
        self.refresh_metadata();
    }

    // Runs process to read apps used track_list metadata async'ly
    pub fn refresh_metadata(&mut self) {
        println!("called read metadata");
        // Set active so the reciever knows to be listening for transmissions
        self.song_loading.active = true;
        self.song_loading.position = 0;
        // clone mpsc sender to move into thread
        let tx = self.song_loading.tx.clone();
        
        // Threaded song metadata reading
        // First we need a copy of every file path we already have
        let paths = {
            let mut paths = vec![];
            self.track_list.iter().for_each(|x| {
                paths.push((*x.path).clone());
            });
            paths
        };
        thread::spawn(move || {
            paths.iter().for_each(|x| {
                // println!("Thread: Reading {}", x);
                tx.send(read_metadata(x)).unwrap();
            });
        });
    }
}

impl eframe::App for SpogApp {
    // runs every time the screen updates, ie every frame
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));
        // println!("R: {}, G: {}, B: {}", self.color_data.elapsed.as_secs(), self.color_data.widget_color.g(), self.color_data.widget_color.b());
        
        // Manually request repaint after 1 second, otherwise the next song will not play until the window is interacted with
        // ctx.request_repaint_after(std::time::Duration::from_secs(1));
        if self.settings.color_shift {
            self.update_color();
            // ctx.request_repaint();
        }

        // (poorly) manually implemented scope that only runs once
        if !self.filter_data.ran {
            let pixels_per_point = ctx.fonts().pixels_per_point();
            let max_texture_side = ctx.fonts().max_texture_side();

            self.filter_data.fonts = egui::text::Fonts::new(
                pixels_per_point, 
                max_texture_side, 
                egui::FontDefinitions::default()
            );
            
            self.refresh_metadata();
            self.filter_data.ran = true;
        }
        if self.song_loading.request {
            self.refresh_directory();
            self.song_loading.request = false;
        }

        // Now recieve sent data
        if self.song_loading.active {
            // println!("{}", self.song_loading.position);
            ctx.request_repaint();
            if let Ok((title, artist, album)) = self.song_loading.rx.try_recv() {
                // println!("Main: Recieved [Title: {}, Artist: {}, Album: {}] in {}", &title, &artist, &album, self.song_loading.position);
                self.track_list[self.song_loading.position].title = Rc::new(title);
                self.track_list[self.song_loading.position].artist = Rc::new(artist);
                self.track_list[self.song_loading.position].album = Rc::new(album);
                self.song_loading.position += 1;
            }
            if self.song_loading.position == self.track_list.len() {
                println!("Main: Finished reading, deactivating...");
                self.song_loading.active = false;
            }
        }

        // Settings window, closed by default
        settings_window(self, ctx, frame);

        // if the song playing ends, get the next song to play and play it
        // This doesn't work because the sink does not register as empty until repainted
        // No easy way to solve this other than requesting repaint every second or so
        if self.on & self.sink.empty() {
            self.fetch_next_song();
            // let sink = self.sink.clone();
            self.run_track();
            // run_track(sink, &self.track_list[self.current_index].path, &mut self.threads);
        }

        if self.settings.mini_mode {
            frame.set_window_size(frame.info().window_info.size.clamp(vec2(600.0, 200.0), vec2(600.0, 200.0)));
            // mini_mode(ctx, frame, self);
            portable_layout(ctx, self);
        } else {
            if frame.info().window_info.size.x < 800.0 || frame.info().window_info.size.y < 600.0 {
                frame.set_window_size(frame.info().window_info.size.clamp(vec2(800.0, 600.0), egui::Vec2::INFINITY));
            }
            // full_app_layout(ctx, self);
            full_layout(ctx, self);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Saves config info on exit
        let roaming_path = dirs::config_dir();
        if let Some(roaming_path) = roaming_path {
            let mem_path = PathBuf::from(format!("{}/{}/{}", roaming_path.as_path().to_str().unwrap(), "SpogifyMem", "config.json"));
            if mem_path.exists() {
                let save = SpogMem {
                    dir_path: self.dir_path.clone(), 
                    current_index: self.current_index, 
                    volume: self.volume,
                    queue_mode: self.mode,
                    volume_mode: self.settings.volume_mode,
                    favorites: self.settings.favorites.clone(),
                    saved_dirs: self.settings.saved_dirs.clone(),
                };
                let ser_save = serde_json::to_string(&save);
                if let Ok(json_contents) = ser_save {
                    std::fs::write(mem_path, json_contents);
                }
            }
        }
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

// Returns an index for the next song based on QueueMode
// maybe move to impl SpogApp, but would need another fn for queueing future songs
pub fn queue_track(state: &QueueMode, index: usize, num_songs: usize) -> usize {
    match state {
        Shuffle => get_rand_track(num_songs),
        Next => get_next_track(index, num_songs),
        Loop => index,
    }
}

// Gets a random song
pub fn get_rand_track(num_songs: usize) -> usize {
    let mut rng = rand::thread_rng();
    let new_index = rng.gen_range(0..num_songs);
    new_index
}

// Gets the next song
pub fn get_next_track(index: usize, num_songs: usize) -> usize {
    let new_index: usize;
    if index < num_songs {
        new_index = index + 1;
    } else {
        new_index = 0;
    }
    new_index
}

// Unfinished, maybe launch player without reading metadata, read it in separate thread, and use it when it's done
pub fn init_song(file_path: &str, file_name: &str, index: usize) -> Song {
    // let path = format!(r"{}\\{}", dir_path, file_name);

    // match format {
    //     FileType::m4a => println!("m4a"),
    //     FileType::mp3 => println!("mp3"),
    //     FileType::Unknown => println!("Unknown")
    // }

    Song {
        title: Rc::new("".to_string()),
        artist: Rc::new("".to_string()),
        album: Rc::new("".to_string()),
        file_name: Rc::new(file_name.to_string()),
        path: Rc::new(file_path.to_string()),
        index,
    }
}

// Read metadata and assign file type/extension
pub fn read_metadata(file_path: &String) -> (String, String, String) {
    // First, need extension so we try to read the correct encoding
    let extension = infer::get_from_path(file_path);
    match extension {
        Ok(ext) => {
            if let Some(t) = ext {
                match t.extension() {
                    "mp3" => read_mp3(file_path),
                    "mp4" => read_m4a(file_path),
                    _ => ("".to_string(),"".to_string(),"".to_string())
                }
            } else {
                ("".to_string(),"".to_string(),"".to_string())
            }
        },
        Err(_) => ("".to_string(),"".to_string(),"".to_string())
    }
}

// Take a song file path and try to return title, artist, and album
// *** This is for m4a encoding
pub fn read_m4a(dir_path: &str) -> (String, String, String) {
    let tag = mp4ameta::Tag::read_from_path(&dir_path);
    match tag {
        Ok(tag) => {
            let title = match tag.title() {
                Some(n) => n.to_string(),
                _ => "".to_string()
            };
            let artist = match tag.artist() {
                Some(n) => n.to_string(),
                _ => "".to_string()
            };
            let album = match tag.album() {
                Some(n) => n.to_string(),
                _ => "".to_string()
            };
            (title, artist, album)
        },
        Err(_) => {
            ("".to_string(), "".to_string(), "".to_string())
        }
    }
}

// mp3 metadata reading
pub fn read_mp3(dir_path: &str) -> (String, String, String) {
    use id3::TagLike;
    let tag = id3::Tag::read_from_path(Path::new(dir_path));
    match tag {
        Ok(tag) => {        
            let title = match tag.title() {
                Some(n) => n.to_string(),
                _ => "".to_string()
            };
            let artist = match tag.artist() {
                Some(n) => n.to_string(),
                _ => "".to_string()
            };
            let album = match tag.album() {
                Some(n) => n.to_string(),
                _ => "".to_string()
            };
            (title, artist, album)
        },
        Err(_) => {
            ("".to_string(), "".to_string(), "".to_string())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpogMem {
    dir_path: String, // Directory last used
    current_index: usize, // Index last used
    volume: f32, // Volume when closed
    queue_mode: QueueMode, // Queue mode when closed ***I THINK doesn't work sometimes
    volume_mode: bool, // Lin or exp volume slider, doesn't work yet so doesn't do anything
    favorites: Vec<usize>, // Favorited songs, will need to make this more robust to work with changing directories
    saved_dirs: Vec<String>, // Saved directories for easy switching
}

impl SpogMem {
    pub fn default() -> Self {
        SpogMem {
            dir_path: r"D:\Music\playlist\music".to_string(), 
            current_index: 0, 
            volume: 0.05,
            queue_mode: QueueMode::Next,
            volume_mode: false,
            favorites: vec![],
            saved_dirs: vec![r"D:\Music\playlist\music".to_string()]
        }
    }
}

pub fn read_mem() -> SpogMem {
    let roaming_path = dirs::config_dir();
    match roaming_path {
        Some(roaming_path) => {
            println!("{}", roaming_path.display());
            let mem_path = PathBuf::from(format!("{}/{}/{}", roaming_path.as_path().to_str().unwrap(), "SpogifyMem", "config.json"));
            if mem_path.exists() {
                println!("mem found");
                let file = std::fs::read_to_string(&mem_path).unwrap();
                if let Ok(t) = serde_json::from_str::<SpogMem>(&file) {
                    t
                } else {
                    let json_contents = serde_json::to_string(&SpogMem::default()).unwrap();
                    std::fs::write(mem_path, json_contents);
                    SpogMem::default()
                }
            } else if mem_path.parent().unwrap().exists() {
                println!("I'll create mem");
                let json_contents = serde_json::to_string(&SpogMem::default()).unwrap();
                std::fs::write(mem_path, json_contents);
                SpogMem::default()
            } else {
                println!("I'll create a config folder");
                SpogMem::default()
            }
        },
        None => {
            SpogMem::default()
        }
    }
}
