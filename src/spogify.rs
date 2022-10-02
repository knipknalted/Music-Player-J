#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::{fs::{read_dir,DirEntry, File},thread::{self, JoinHandle}, 
    io::{BufReader},sync::Arc, collections::VecDeque, time::{Duration, SystemTime}, f64::consts::PI, rc::Rc, path::{Path, PathBuf}};
use rodio::Decoder;
use rand::{self, Rng};
use egui::vec2;
//use id3::{Tag, TagLike};
use mp4ameta::Tag;
use serde::{Serialize, Deserialize};
use thin_vec::{thin_vec, ThinVec};
use crate::{controls::{settings_window,LIGHT_GREY, SLIDER_BACKGROUND}, portable::portable_layout};
use crate::full::full_layout;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum QueueState {
    Shuffle, // The next song played should be randomly selected from the available songs
    Next, // The next song played should be the next song in the list
    Loop, // The next song played should be the same song
}

use QueueState::{Shuffle,Next,Loop};

pub struct Song {
    pub title: Rc<String>,    // name/title of the song
    pub artist: Rc<String>,  // artist name for the song
    pub album: Rc<String>,   // album name for the song
    pub file_name: Rc<String>, // file name
    pub path: Rc<String>,    // file path to the song
    pub index: usize,    // number to keep track of what the index in the list is for the song, for convenience
}

impl Song {
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
        use egui::Align;
        // If there's any metadata, use an actual label, otherwise just show the path
        if complete_song {
            // Positions for title, artist, and album text
            // let title_pos = rect.left_center() + vec2(20.0 + 45.0, 0.0);
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
            // let galley = ui.fonts().layout_job(job);
            ui.painter().galley(first_pos - vec2(0.0, 0.5*name_height), name_gal);
        }
        card_response
        // ui.painter().rect_filled(rect, egui::Rounding::same(3.0), SLIDER_BACKGROUND);
    }
}

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
        
        FilterData {field: String::new(), active: false, text_width: 0.0, ran: false, fonts}
    }
}

pub struct Settings {
    pub color_shift: bool, // Is fancy changing rgb colored buttons on
    pub muted: bool, // Is the sink muted
    pub direct_buf: String, // Buffer for directory changes
    pub mini_mode: bool, // Whether the app is in mini_mode or not
    pub volume_mode: bool, // Whether the volume slider is logarithmic or linear
}

pub struct SpogApp {
    pub direct_path: String, // the location of the folder being used
    pub track_list: Vec<Song>, //the list of songs being used
    pub on: bool, // Whether the stream is paused or not
    pub finished: bool,
    pub volume: f32, // Stream volume
    pub mode: QueueState, // Shuffle, loop, etc.
    pub current_index: usize, // index of the current song, used to 
    pub past_songs: VecDeque<usize>, // store indices of past n songs
    pub stream: rodio::OutputStream, // stream because it broke when I took it out
    pub stream_handle: rodio::OutputStreamHandle, // handle to the stream for resetting the sink
    pub sink: Arc<rodio::Sink>, // Atomic reference to a sink, so it's thread safe
    pub threads: ThinVec<JoinHandle<()>>, // temporary place to hold handles to threads, so they can be explicitly dropped, otherwise threads are never closed
    pub window_bools: WindowBools, // bools for if windows are open
    pub color_data: RgbShiftData, // Info using elapsed time to make fancy changing colors might take it out
    pub filter_data: FilterData, // Info used to render and use custom search bar
    pub settings: Settings, // Bools for which features are turned on, like color shift
    pub play_dur: SystemTime // Time set every time we call run_track, used when going to previous track
    // pub profiler: Option<dhat::Profiler>, // To easily switch between usuing a profiler and not
}

impl SpogApp {
    // create a new app with some default settings, given a folder path to start from
    pub fn new(cc: &eframe::CreationContext) -> SpogApp {
        let mem: SpogMem = read_mem();

        // Set the app to initialize with dark mode and other custom widget settings
        let visuals: egui::Visuals = egui::Visuals::dark();      
        cc.egui_ctx.set_style(egui::Style{visuals, spacing: egui::style::Spacing{slider_width: 200.0, ..Default::default()}, ..Default::default()});

        let direct_path = String::from(&mem.dir_path); // What directory path are we using
        let track_names = get_tracks(&direct_path); // Get all file paths from that directory
        
        // Make an empty list of songs
        let mut track_list:Vec<Song> = Vec::with_capacity(track_names.len());
        // Then for each file path, see if it's a valid song, and add it to the list of songs if it is
        for i in 0..track_names.len() {
            if let Some(t) = get_song(direct_path.as_str(), track_names[i].as_str(), i) {
                track_list.push(t)
            }
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
        };
        let play_dur = SystemTime::now();
        SpogApp {direct_path, 
            track_list, 
            on: false, 
            finished: false,
            volume: mem.volume, 
            mode: mem.queue_mode,
            current_index, 
            past_songs, 
            stream, 
            stream_handle, 
            sink, 
            threads,
            window_bools,
            color_data,
            filter_data,
            settings,
            play_dur,
            // profiler: None,
        }    
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
        let next_index = queue_track(&self.mode, self.current_index, self.track_list.len());
        self.current_index = next_index;
    }

    // showing a button for every song 
    pub fn render_play_buttons(&mut self, ui: &mut egui::Ui) {
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
                    }
                });
            };
        }
    }

    // Take a sink, track_path, and thread vec (to drop the thread handle when we're done) and play the song 
    pub fn run_track(&mut self) {
        self.play_dur = SystemTime::now();
        self.finished = false;
        let file = BufReader::with_capacity(8000, File::open(&*self.track_list[self.current_index].path).unwrap());
        // println!("{}", std::fs::read(&*self.track_list[self.current_index].path).unwrap().len());
        let source = Decoder::new(file).expect(format!("Panicked at {}", self.current_index).as_str());
        
        // match source.total_duration() {
        //     Some(t) => println!("This song is {} seconds long", t.as_secs()),
        //     None => println!("Couldn't read the duration on this song"),
        // }

        // let sink = self.sink.clone();
        // let song_thread = thread::spawn(move || {        
        //     sink.append(source);
        // });
        // self.threads.push(song_thread);

        thread::scope(|s| {
            let _song_thread = s.spawn(|| {
                self.sink.append(source);
                // self.sink.sleep_until_end();
                // thread::sleep(Duration::from_secs(5));
                // self.finished = true;
            });
            // self.threads.push(song_thread);
        });
        
    }

    pub fn set_sink(&mut self) {
        self.sink = Arc::new(rodio::Sink::try_new(&self.stream_handle).unwrap());
        // self.sink = Arc::new(rodio::Sink::new_idle().0);
        self.sink.set_volume(self.volume);
        // self.sink.set_speed(16.0);
    }

    pub fn skip_song(&mut self) {
        self.fetch_next_song();
        self.threads.clear();
        self.sink.stop();
        self.set_sink();
        // let sink = self.sink.clone();
        // run_track(sink, &self.track_list[self.current_index].path, &mut self.threads);
        self.run_track()
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

    pub fn _stress_test(&mut self) {
        // stress test conditions
        // go to next song using a randomly selected mode
        // play for randomly set duration before proceeding
        let mut rng = rand::thread_rng();
        let mode = rng.gen_range(0..9);
        match mode {
            0..=6 => self.mode = Shuffle,
            7 => self.mode = Next,
            _ => self.mode = Loop,
        }
        let test_dur = rng.gen_range(0..10);

        self.skip_song();
        std::thread::sleep(std::time::Duration::from_secs(test_dur));
        self.threads.clear();

    }

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
}

impl eframe::App for SpogApp {
    // runs every time the screen updates, ie every frame
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // println!("R: {}, G: {}, B: {}", self.color_data.elapsed.as_secs(), self.color_data.widget_color.g(), self.color_data.widget_color.b());
        
        // Manually request repaint after 1 second, otherwise the next song will not play until the window is interacted with
        // ctx.request_repaint_after(std::time::Duration::from_secs(1));
        if self.settings.color_shift {
            self.update_color();
            // ctx.request_repaint();
        }
        if !self.filter_data.ran {
            let pixels_per_point = ctx.fonts().pixels_per_point();
            let max_texture_side = ctx.fonts().max_texture_side();

            self.filter_data.fonts = egui::text::Fonts::new(
                pixels_per_point, 
                max_texture_side, 
                egui::FontDefinitions::default()
            );
            self.filter_data.ran = true;
        }

        // Settings window, closed by default
        settings_window(self, ctx, frame);
        // settings_area(self, ctx, frame);

        // if the song playing ends, get the next song to play and play it
        ctx.request_repaint_after(Duration::from_secs(1));
        if self.on & self.sink.empty() {
            // ctx.request_repaint();
            // println!("Sink is empty");
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
        let roaming_path = dirs::config_dir();
        if let Some(roaming_path) = roaming_path {
            let mem_path = PathBuf::from(format!("{}/{}/{}", roaming_path.as_path().to_str().unwrap(), "SpogifyMem", "config.json"));
            if mem_path.exists() {
                let save = SpogMem {
                    dir_path: self.direct_path.clone(), 
                    current_index: self.current_index, 
                    volume: self.volume,
                    queue_mode: self.mode,
                    volume_mode: self.settings.volume_mode,
                };
                let ser_save = serde_json::to_string(&save);
                if let Ok(json_contents) = ser_save {
                    std::fs::write(mem_path, json_contents);
                }
            }
        }
    }
}

pub fn get_tracks(dir_path: &str) -> Vec<String> {
    let mut file_vec: Vec<DirEntry> = vec![];
    
    // First we get each file in the file directory
    read_dir(dir_path).unwrap().for_each(|file| {
        // println!("{}", file_vec.capacity());
        match file {
            Ok(n) => file_vec.push(n),
            Err(e) => println!("oh fuck oh shit this broke it {}", e),
        }
    });

    let mut file_names: Vec<String> = Vec::with_capacity(file_vec.len());

    // Then we get the name of each file and format it into the file's path
    file_vec.into_iter().for_each(|file| {
        file_names.push(file.file_name().into_string().unwrap())
        
    });
    return file_names;
}

pub fn queue_track(state: &QueueState, index: usize, num_songs: usize) -> usize {
    match state {
        Shuffle => get_rand_track(num_songs),
        Next => get_next_track(index, num_songs),
        Loop => index,
    }
}

pub fn get_rand_track(num_songs: usize) -> usize {
    let mut rng = rand::thread_rng();
    let new_index = rng.gen_range(0..num_songs);
    new_index
}

pub fn get_next_track(index: usize, num_songs: usize) -> usize {
    let new_index: usize;
    if index < num_songs {
        new_index = index + 1;
    } else {
        new_index = 0;
    }
    new_index
}

// Take a song file's path and try to turn it into a Song
pub fn get_song(dir_path: &str, song_name: &str, index: usize) -> Option<Song> {

    let file_name = Rc::new(String::from(song_name));
    let path = Rc::new(String::from(format!(r"{}\\{}", dir_path, song_name)));   
    let tag = Tag::read_from_path(path.as_str());
    
    let result: Option<Song> = match tag {
        Ok(tag) => {
            let (title, artist, album) = if !tag.is_empty() {
                (
                    match tag.title() {
                    Some(n) => Rc::new(n.to_string()),
                    None => Rc::new(String::new())
                    },
                    match tag.artist() {
                        Some(n) => Rc::new(n.to_string()),
                        None => Rc::new(String::new())
                    },
                    match tag.album() {
                        Some(n) => Rc::new(n.to_string()),
                        None => Rc::new(String::new())
                    }
                )
            } else {
                (
                    Rc::new(String::new()),
                    Rc::new(String::new()),
                    Rc::new(String::new())
                )
            };
            let song = Song {title, artist, album, file_name, path, index};
            Some(song)
        },
        Err(_) => None,
    };
    result
}

#[derive(Serialize, Deserialize)]
pub struct SpogMem {
    dir_path: String,
    current_index: usize,
    volume: f32,
    queue_mode: QueueState,
    volume_mode: bool
}

impl SpogMem {
    pub fn default() -> Self {
        SpogMem {
            dir_path: r"D:\Music\playlist\music".to_string(), 
            current_index: 0, 
            volume: 0.05,
            queue_mode: QueueState::Next,
            volume_mode: false,
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
