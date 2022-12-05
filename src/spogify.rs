use std::{fs::File,thread::{self, JoinHandle}, 
    io::BufReader,sync::{Arc, mpsc::{Sender, Receiver}}, collections::{VecDeque, HashMap}, time::{Duration, SystemTime}, 
    rc::Rc, path::{PathBuf, Path}, cmp::{Reverse, Eq, PartialEq,}
};

use colorgrad::Gradient;
use cpal::{Device, traits::HostTrait};
use lofty::{AudioFile, Accessor};
use magnum::container::ogg::OpusSourceOgg;
use rodio::{Decoder, Source, DeviceTrait};
use rand::{self, Rng};
use egui::{vec2};
use serde::{Serialize, Deserialize};
use thin_vec::{thin_vec, ThinVec};
use time::OffsetDateTime;
use crate::{controls::{settings_window}, portable::portable_layout, full::full_layout, song::CachedSong};
use crate::song::{Song, init_dir};

// Duration of one full loop through color gradient
const COLOR_LOOP_DUR: f32 = 20.0;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum QueueMode {
    Shuffle, // The next song played should be randomly selected from the available songs
    Next, // The next song played should be the next song in the list
    Loop, // The next song played should be the same song
}

// List of valid formats in looking for files, might want to refactor
// const FORMATS: [&'static str; 2] = ["mp3", "mp4"];

use QueueMode::{Shuffle,Next,Loop};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SupportedFormat {
    m4a,
    mp3,
    // opus,
    flac,
    aac,
    ogg,
    opus,
}

impl std::fmt::Display for SupportedFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::m4a => write!(f, "m4a"),
            Self::mp3 => write!(f, "mp3"),
            // Self::opus => write!(f, "opus"),
            Self::flac => write!(f, "flac"),
            Self::aac => write!(f, "aac"),
            Self::ogg => write!(f, "ogg"),
            Self::opus => write!(f, "opus"),
        }
    }
}

pub fn split_artists(artist_str: &str) -> Vec<Rc<String>> {
    let artist_vec: Vec<Rc<String>> = {
        let buf = artist_str.split(',').collect::<Vec<&str>>();
        let mut vec = vec![];
        for i in 0..buf.len() {
            vec.push(Rc::new(buf[i].trim().to_string()));
        }
        vec
    };
    artist_vec
}

pub struct WindowBools {
    pub directory: bool, // Is the directory window open ***NO LONGER NEEDED
    pub settings: bool, // Is the settings window open
}

pub struct RgbShiftData {
    pub now: SystemTime, // Reference time
    pub grad: Gradient, // color gradient for color shift
    pub position: f64,
    pub elapsed: Duration, // Elapsed time since self.now, maybe unnecessary
    pub widget_color: egui::Color32, // rgb color for widgets
    pub widget_detail_color: egui::Color32, // color for details ex. text inside a widget, for if I want to make it reactive
}

pub struct FilterData {
    pub field: String, // What is currently typed in the search bar
    pub position: usize, // Where is the caret in the search bar(maybe Option<usize> instead?)
    pub active: bool, // Is the search bar active
    pub selected_j: Option<usize>, // If the filter is active and isn't empty, which song is selected by arrow keys
    pub track_i: Option<usize>, // Corresponding track number
    pub text_width: f32, // How much wide is the text in the search bar taking up (so I know where to put the cursor)
    pub ran: bool, // Used to make sure we only remake self.fonts once, probably unnecessary, remove later
    pub fonts: egui::text::Fonts, // The font info we need to figure out and update self.text_width
    pub blink_timer: SystemTime, // To make the caret blink
}

impl FilterData {
    pub fn default() -> Self {
        let fonts = egui::text::Fonts::new(
            1.0, 
            40000, 
            egui::FontDefinitions::default()
        );
        
        FilterData {
            field: String::new(), 
            position: 0, 
            active: false, 
            selected_j: None, 
            track_i: None, 
            text_width: 0.0, 
            ran: false, 
            fonts,
            blink_timer: SystemTime::now(),
        }
    }
}

pub struct Sorting {
    pub alphebetical: bool, // Sorted alphabetically(true) or reverse alphabetically(false)
    pub column: Column,
}

impl Sorting {
    pub fn default() -> Self {
        Sorting {alphebetical: true, column: Column::FileName}
    }
}

// Which parameter is being used for sorting
#[derive(Clone, Copy, PartialEq)]
pub enum Column {
    Title,
    Artist,
    Album,
    Date,
    FileName
}

// Data for a simple in-app tool to execute and pass arguments to yt-dlp
pub struct DownloadConfig {
    pub dlp_path: String,
    pub target_dir: String, // Where downloaded files should go
    pub url: String, // URL
    pub format: String,  
}

impl DownloadConfig {
    pub fn default() -> Self {
        DownloadConfig {
            dlp_path: r"D:\Music\playlist\yt-dlp.exe".to_string(),
            target_dir: r"D:\Music\playlist\temp".to_string(), 
            url: "".to_string(), 
            format: "".to_string()
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum SliderMode {
    Linear,
    Exponential,
}

pub struct Settings {
    pub cache_dir: bool, // Should we try to cache song metadata on close
    pub color_shift: bool, // Is fancy changing rgb colored buttons on
    pub muted: bool, // Is the sink muted
    pub direct_buf: String, // Buffer for directory changes
    pub mini_mode: bool, // Whether the app is in mini_mode or not
    pub volume_mode: SliderMode, // Whether the volume slider is exponential or linear
    pub favorites: Vec<usize>, // List of favorited song indices *** Switch to path or metadata to keep track
    pub saved_dirs: Vec<String>, // Saved directories for easy switching
    pub sorting: Sorting, // How the song list is being sorted (file name A-Z on launch)
    pub volume_range: (f32, f32), // (Lower, Upper) bounds on volume slider
    pub volume_buf: (String, String), // Buffer for volume slider changes in settings textedit field
    pub download_config: DownloadConfig, // Info to use built-in downloader
    pub num: usize, 
    pub ref_time: SystemTime,
    pub device_index: usize, // Index of current device in devices vec
    // pub volume_knob_ref: f32,
}

pub struct PlaybackInfo {
    pub start_time: SystemTime, // Set when the current song starts playing, required by some features
    pub speed: f32, // Sink speed
    pub elapsed_time: u128, // in millis
    pub cached_time: u128, // in millis, used to keep track of time wrt pauses
    pub hidden_queue: usize, // Backend queue used to speed up changing songs, set for next song after playing a song
}

impl PlaybackInfo {
    fn default() -> Self {
        let start_time = SystemTime::now();
        let speed = 1.0;
        let hidden_queue = 0;
        let elapsed_time = 0;
        let cached_time = 0;
        PlaybackInfo {start_time, speed, elapsed_time, cached_time, hidden_queue}
    }
}

#[derive(Clone)]
pub struct SongData {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub date: OffsetDateTime,
    pub duration: Option<Duration>
}

impl SongData {
    fn default() -> Self {
        Self { 
            title: "".to_string(), 
            artist: "".to_string(), 
            album: "".to_string(), 
            date: OffsetDateTime::UNIX_EPOCH, 
            duration: None 
        }
    }

    fn from_duration(duration: Duration) -> Self {
        Self { 
            title: "".to_string(), 
            artist: "".to_string(), 
            album: "".to_string(), 
            date: OffsetDateTime::UNIX_EPOCH, 
            duration: Some(duration), 
        }
    }
}

pub struct SongLoad {
    pub request: bool, // Set to true if need to ask app to read metadata from other scopes
    pub active: bool, // Easy toggle on/off to reduce conditionals in use
    pub tx: Sender<SongData>, // Sender for title, artist, album
    pub rx: Receiver<SongData>, // Receiver for title, artist, album
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
    pub window_bools: WindowBools, // ***Move to settings*** Bools for if windows are open
    pub color_data: RgbShiftData, // Info using elapsed time to make fancy changing colors might take it out
    pub filter: FilterData, // Info used to render and use custom search bar
    pub settings: Settings, // Various app info/config data and low-use data
    pub playback: PlaybackInfo, // Song playback info *move volume here eventually
    pub song_loading: SongLoad, // sender and reciever used to read song metadata in separate thread
    pub devices: Vec<Device>, // List of output devices available
}

impl SpogApp {
    // create a new app with some default settings, given a folder path to start from
    pub fn new(cc: &eframe::CreationContext) -> SpogApp {
        let mem: SpogMem = read_mem(); // TODO save output device to memory

        let host = cpal::default_host();
        let devices: Vec<Device> = if let Ok(devices) = host.output_devices() {
            devices.collect()
        } else { vec![] };
        
        // Get default device and find its index in list of devices
        // ***DEFINITELY A BETTER WAY TO HANDLE THIS INITIALIZATION
        let default_device = host.default_output_device().unwrap();
        let device_index = {
            devices.iter().position(|x| x.name().unwrap() == default_device.name().unwrap()).unwrap()
        };

        // Set the app to initialize with dark mode and other custom widget settings
        let visuals: egui::Visuals = egui::Visuals::dark();      
        cc.egui_ctx.set_style(egui::Style{visuals, spacing: egui::style::Spacing{slider_width: 200.0, ..Default::default()}, ..Default::default()});

        let dir_path = String::from(&mem.dir_path); // What directory path are we using
        
        // Get any valid audio files in directory without reading metadata
        let mut track_list = init_dir(&dir_path);

        // Use cached metadata if enabled
        if mem.cache_dir {
            track_list.iter_mut().for_each(|x| {
                if mem.cache.contains_key(&(*x.path)) {
                    x.title = Rc::new(mem.cache[&(*x.path)].title.clone());
                    x.artist = Rc::new(mem.cache[&(*x.path)].artist.clone());
                    x.album = Rc::new(mem.cache[&(*x.path)].album.clone());
                }
            });
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
            grad: colorgrad::sinebow(),
            position: 0.0,
            elapsed: Duration::new(0, 0), 
            widget_color: egui::Color32::from_rgb(104, 185, 115),
            widget_detail_color: egui::Color32::WHITE,
        };

        let filter_data = FilterData::default();
        
        let settings = Settings {
            cache_dir: mem.cache_dir,
            color_shift: false, 
            muted: false, 
            direct_buf: String::new(), 
            mini_mode: false,
            volume_mode: mem.volume_mode,
            favorites: mem.favorites,
            saved_dirs: mem.saved_dirs,
            sorting: Sorting::default(),
            volume_range: (0.0, 0.2),
            volume_buf: ("0.0".to_string(), 0.2.to_string()),
            download_config: DownloadConfig::default(),
            num: 0,
            ref_time: SystemTime::now(),
            device_index,
        };

        let playback = PlaybackInfo::default();

        let (tx, rx) = std::sync::mpsc::channel::<SongData>();
        let song_loading = SongLoad {request: false, active: false, tx, rx, position: 0,};

        let mut app = SpogApp {
            dir_path,
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
            filter: filter_data,
            settings,
            playback,
            song_loading,
            devices,
        };
        // Starts the current song on the sink, otherwise the song on launch will be skipped(never given to the sink)
        if !app.track_list.is_empty() {
            app.run_track(&cc.egui_ctx);
            app.pause();
        }
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

    pub fn playback_update(&mut self) {
        if self.on {
            self.playback.elapsed_time = self.playback.cached_time + self.playback.start_time.elapsed().unwrap().as_millis();
        }
    }

    pub fn play(&mut self) {
        self.playback.start_time = SystemTime::now();
        self.sink.play();
    }

    pub fn pause(&mut self) {
        self.playback.cached_time += self.playback.start_time.elapsed().unwrap().as_millis();
        self.sink.pause();
    }

    // showing a button for every song 
    pub fn render_play_buttons(&mut self, ui: &mut egui::Ui, current_response: egui::Response) {
        const CARD_HEIGHT: f32 = 50.0;
        let offset_nw = ui.available_rect_before_wrap().left_top();
        let offset_ne = ui.available_rect_before_wrap().right_top();
        let mut j: f32 = 0.0;
        // let mut selected_buf: Option<usize> = None;

        for i in 0..self.track_list.len() {
            let title = &self.track_list[i].title;
            let artist = &self.track_list[i].artist;
            let name = &self.track_list[i].file_name;

            // BIG LOGICAL MUMBO BREAKDOWN
            // '||' means OR, but it doesn't keep going if it's already true, '|' means regular OR
            // First, if the filter is empty, skip the other conditions and just show the card
            // If the filter isn't empty, then check for partial matches with the song title, artist, or album
            // Also check if the filter exactly matches the title (I don't think I need this, will come back to it)
            if self.filter.field.is_empty() || title.to_ascii_uppercase().contains(&self.filter.field.to_ascii_uppercase()) |
                artist.to_ascii_uppercase().contains(&self.filter.field.to_ascii_uppercase()) |
                name.to_ascii_uppercase().contains(&self.filter.field.to_ascii_uppercase()) || *self.filter.field == **title 
            {
                let card_nw = offset_nw + vec2(10.0, (j as f32)*(CARD_HEIGHT+10.0));
                let card_se = offset_ne + vec2(-10.0, CARD_HEIGHT + (j as f32)*(CARD_HEIGHT+10.0));
                let rect = egui::Rect::from_two_pos(card_nw, card_se);

                if let Some(num) = self.filter.selected_j {
                    if num == j as usize {
                        self.filter.track_i = Some(i);
                    }
                }

                j += 1.0;
                ui.allocate_ui_at_rect(rect, |ui| {
                    if ui.is_rect_visible(rect) {
                        let response = self.track_list[i].render_card(
                            ui, 
                            rect, 
                            i,
                            j as usize-1, 
                            self.current_index, 
                            self.filter.selected_j
                        );

                        if response.clicked() {
                            // self.threads.clear();
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
                            self.run_track(ui.ctx());
                        }
                        response.context_menu(|ui| {
                            ui.label(format!("Format: {}", self.track_list[i].format));
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
                            }
                        });
                    }
                    
                    // If the current song display is clicked, scroll to that song in the list
                    if current_response.clicked() && i == self.current_index {
                        ui.scroll_to_rect(rect, Some(egui::Align::Center));
                    }
                });
            }
        }

        current_response.context_menu(|ui| {
            ui.label(format!("Format: {}", self.track_list[self.current_index].format));
            if ui.button("Queue Song").clicked() {
                self.queued_song = Some(self.current_index);
            }
            if ui.button("Favorite").clicked() {
                if self.settings.favorites.contains(&self.current_index) {
                    let index = self.settings.favorites.iter().position(|index| *index == self.current_index).unwrap();
                    self.settings.favorites.swap_remove(index);
                } else {
                    self.settings.favorites.push(self.current_index);
                }
            }
        });
        
        // If there is a song selected with arrow keys, play it
        // if let Some(i) = self.filter.track_i {
        //     // println!("{i}");

        //     for event in &ui.ctx().input().events {
                
        //         if let egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers: _ } = event {
        //             // println!("got enter");
        //             self.sink.stop();
        //             self.set_sink();
        //             self.on = true;

        //             // Put the previous song into history before playing the one clicked on
        //             if self.past_songs.len() == Some(20).unwrap() {
        //                 self.past_songs.pop_back();
        //             }
        //             self.past_songs.insert(0, self.current_index);
        //             // set the current index to be that of the song clicked on
        //             self.current_index = i;
        //             self.run_track(ui.ctx());
        //         }
        //     }
        // }
    }

    // Plays song at apps current index, does not handle anything with changing the current index
    pub fn run_track(&mut self, ctx: &egui::Context) {
        self.threads.clear();
        println!("{}", self.track_list[self.current_index].file_name);
        self.playback.start_time = SystemTime::now();
        let file = File::open(&*self.track_list[self.current_index].path);

        let rodio_source = if let Ok(file) = file {
            let file = BufReader::new(file);
            let source = Decoder::new(file);
            match source {
                Ok(t) => Some(t.buffered()),
                Err(_) => None,
            }
        } else {
            None
        };

        let magnum_source: Option<OpusSourceOgg<BufReader<File>>> = if let None = rodio_source {
            if let Ok(file) = File::open(&*self.track_list[self.current_index].path) {
                let file = BufReader::new(file);
                if let Ok(source) = OpusSourceOgg::new(file) {
                    Some(source)
                } else {None}
            } else {None}
        } else {
            None
        };

        
    
        if rodio_source.is_some() || magnum_source.is_some() {
            let blah = ctx.clone();
            let sink = self.sink.clone();
            if let Some(source) = rodio_source {
                let song_thread = thread::spawn(move || {
                    sink.append(source);
                    sink.sleep_until_end();
                    blah.request_repaint();
                });
                self.threads.push(song_thread);
            } else if let Some(source) = magnum_source {
                let song_thread = thread::spawn(move || {
                    sink.append(source);
                    sink.sleep_until_end();
                    blah.request_repaint();
                });
                self.threads.push(song_thread);
            }
                        
            // thread::scope(|s| {
            //     let _song_thread = s.spawn(|| {
            //         self.sink.append(source);
            //     });
            // });
        }
    }

    // Resets the app's sink to play a new song
    pub fn set_sink(&mut self) {
        self.sink = Arc::new(rodio::Sink::try_new(&self.stream_handle).unwrap());
        self.sink.set_volume(self.volume);
        self.sink.set_speed(self.playback.speed)
    }

    // Changes output stream to a different device(given index for self.devices)
    pub fn set_stream(&mut self, index: usize) {
        self.on = false;
        (self.stream, self.stream_handle) = rodio::OutputStream::try_from_device(&self.devices[index]).unwrap();
        self.set_sink();
    }

    // Skips current song
    pub fn skip_song(&mut self, ctx: &egui::Context) {
        self.fetch_next_song();
        self.sink.stop();
        self.set_sink();
        self.run_track(ctx);
        self.on = true;
    }

    pub fn go_back(&mut self, ctx: &egui::Context) {
        // If more than 3 seconds into the song, restart it
        if self.playback.start_time.elapsed().unwrap() > Duration::from_secs(3) {
            if self.playback.start_time.elapsed().unwrap() > Duration::from_secs(3) {
                self.sink.stop();
                self.set_sink();
                self.run_track(ctx);
            }
        } 
        // Otherwise, go to the previous song in history, if there is one
        else if self.past_songs.len() > 0 {
            self.current_index = self.past_songs.pop_front().unwrap();
            self.sink.stop();

            self.set_sink();
            self.run_track(ctx);
        }
    }

    // Updates the color for rgb color shift
    pub fn update_color(&mut self, dt: f32) {
        self.color_data.position += (dt/COLOR_LOOP_DUR) as f64;
        if self.color_data.position > 1.0 {
            self.color_data.position -= 1.0;
        }
        let rgb = self.color_data.grad.at(self.color_data.position).to_rgba8();

        self.color_data.widget_color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    }

    // Synchronously: get valid files in directory and set track_list to it
    // Then Async: (try to) read metadata and pass it to app when ready
    pub fn refresh_directory(&mut self) {
        println!("called refresh directory");

        self.track_list = init_dir(&self.dir_path);

        // Read song metadata
        self.refresh_metadata();
    }

    // Runs process to read apps used track_list metadata async'ly
    // Switch from sending String and OffsetDateTime to Option<...> reduce new unnecessary allocations
    pub fn refresh_metadata(&mut self) {
        println!("called read metadata");
        // Set active so the reciever knows to be listening for transmissions
        self.song_loading.active = true;
        self.song_loading.position = 0;
        // clone mpsc sender to move into thread
        let tx = self.song_loading.tx.clone();
        
        // Threaded song metadata reading
        // First we need a copy of every file path we already have
        let info = {
            let mut paths = vec![];
            self.track_list.iter().for_each(|x| {
                paths.push(((*x.path).clone(), x.format));
            });
            paths
        };
        thread::spawn(move || {
            info.iter().for_each(|x| {
                // println!("Thread: Reading {}", x);
                tx.send(read_metadata(&x.0, x.1)).unwrap();
            });
        });
    }

    pub fn sort_tracklist(&mut self) {
        println!("Called sort tracklist");
        let index_buf = self.track_list[self.current_index].clone();
        // Separate songs for which the sorted field is empty i.e. there is no metadata for it
        let mut temp: Vec<Song> = match self.settings.sorting.column {
            Column::Title => {
                println!("identified title");
                let mut temp = vec![];
                let mut i = 0;
                while i < self.track_list.len() {
                    if self.track_list[i].title.is_empty() {
                        let val = self.track_list.remove(i);
                        temp.push(val);
                    } else {
                        i += 1;
                    }
                }
                temp
            },
            Column::Artist => {
                println!("identified artist");
                let mut temp = vec![];
                let mut i = 0;
                while i < self.track_list.len() {
                    if self.track_list[i].artist.is_empty() {
                        let val = self.track_list.remove(i);
                        temp.push(val);
                    } else {
                        i += 1;
                    }
                }
                temp
            },
            Column::Album => {
                println!("identified album");
                let mut temp = vec![];
                let mut i = 0;
                while i < self.track_list.len() {
                    if self.track_list[i].album.is_empty() {
                        let val = self.track_list.remove(i);
                        temp.push(val);
                    } else {
                        i += 1;
                    }
                }
                temp
            },
            Column::FileName => vec![],
            Column::Date => {
                println!("identified date");
                let mut temp = vec![];
                let mut i = 0;
                while i < self.track_list.len() {
                    if self.track_list[i].date == OffsetDateTime::UNIX_EPOCH {
                        let val = self.track_list.remove(i);
                        temp.push(val);
                    } else {
                        i += 1;
                    }
                }
                temp
            }
        };

        // Sort the remaining songs (that do have metadata)
        match self.settings.sorting.column {
            Column::Title => {
                if self.settings.sorting.alphebetical {
                    // Maybe for title don't remove songs, instead use song.filename if song.title.is_empty()
                    self.track_list.sort_by_key(|x| x.title.to_owned().to_ascii_uppercase())
                } else {
                    self.track_list.sort_by_key(|x| Reverse(x.title.to_owned().to_ascii_uppercase()))
                }
            },
            Column::Artist => {
                if self.settings.sorting.alphebetical {
                    self.track_list.sort_by_key(|x| x.artist.to_owned().to_ascii_uppercase())
                } else {
                    self.track_list.sort_by_key(|x| Reverse(x.artist.to_owned().to_ascii_uppercase()))
                }
            },
            Column::Album => {
                if self.settings.sorting.alphebetical {
                    self.track_list.sort_by_key(|x| x.album.to_owned().to_ascii_uppercase())
                } else {
                    self.track_list.sort_by_key(|x| Reverse(x.album.to_owned().to_ascii_uppercase()))
                }
            },
            Column::FileName => {
                if self.settings.sorting.alphebetical {
                    self.track_list.sort_by_key(|x| x.file_name.to_owned())
                } else {
                    self.track_list.sort_by_key(|x| Reverse(x.file_name.to_owned()))
                }
            },
            Column::Date => {
                if self.settings.sorting.alphebetical {
                    self.track_list.sort_by_key(|x| x.date)
                } else {
                    self.track_list.sort_by_key(|x| Reverse(x.date))
                }
            }
        }
        // Add back the songs without metadata at the end, so they will never be at the top of the list when sorted
        self.track_list.append(&mut temp);
        // Adjust app's current index to be the new index of the same file
        self.current_index = self.track_list.iter().position(|x| x == &index_buf).unwrap();
        println!("Finished sorting");
    }
}

impl eframe::App for SpogApp {
    // runs every time the screen updates, ie every frame
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        self.playback_update();
        println!("{}", self.playback.elapsed_time);

        if self.settings.mini_mode {
            frame.set_window_size(frame.info().window_info.size.clamp(vec2(600.0, 200.0), vec2(600.0, 200.0)));
            portable_layout(ctx, self, frame);
        } else {
            if frame.info().window_info.size.x < 800.0 || frame.info().window_info.size.y < 600.0 {
                frame.set_window_size(frame.info().window_info.size.clamp(vec2(800.0, 600.0), egui::Vec2::INFINITY));
            }
            full_layout(ctx, self, frame);
        }
        
        // println!("R: {}, G: {}, B: {}", self.color_data.elapsed.as_secs(), self.color_data.widget_color.g(), self.color_data.widget_color.b());
        
        // Manually request repaint after 1 second, otherwise the next song will not play until the window is interacted with
        // ctx.request_repaint_after(std::time::Duration::from_secs(1));
        if self.settings.color_shift {
            let dt = ctx.input().stable_dt;
            self.update_color(dt);
            ctx.request_repaint();
        }

        // (poorly) manually implemented scope that only runs once
        if !self.filter.ran {
            let pixels_per_point = ctx.fonts().pixels_per_point();
            let max_texture_side = ctx.fonts().max_texture_side();

            self.filter.fonts = egui::text::Fonts::new(
                pixels_per_point, 
                max_texture_side, 
                egui::FontDefinitions::default()
            );
            
            self.refresh_metadata();
            self.filter.ran = true;
        }
        if self.song_loading.request {
            self.refresh_directory();
            self.song_loading.request = false;
        }

        // Now recieve sent data
        if self.song_loading.active {
            // println!("{}", self.song_loading.position);
            ctx.request_repaint();
            if let Ok(song_data) = self.song_loading.rx.try_recv() {
                // println!("Main: Recieved [Title: {}, Artist: {}, Album: {}] in {}", &title, &artist, &album, self.song_loading.position);

                self.track_list[self.song_loading.position].title = Rc::new(song_data.title);
                self.track_list[self.song_loading.position].artist = Rc::new(song_data.artist);
                self.track_list[self.song_loading.position].album = Rc::new(song_data.album);
                self.track_list[self.song_loading.position].date = song_data.date;
                self.track_list[self.song_loading.position].duration = song_data.duration;
                self.song_loading.position += 1;
            }
            if self.song_loading.position == self.track_list.len() {
                println!("Main: Finished reading, deactivating...");
                self.song_loading.active = false;
            }
        }

        // Settings window, closed by default
        settings_window(self, ctx, frame);

        // Works now because pass a reference to ctx to the song thread, sleep until the sink is done, and then request refresh from there
        if self.on & self.sink.empty() {
            println!("sink was empty");
            self.fetch_next_song();
            self.run_track(ctx);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Saves config info on exit
        let roaming_path = dirs::config_dir();
        if let Some(roaming_path) = roaming_path {
            let mem_path = PathBuf::from(format!("{}/{}/{}", roaming_path.as_path().to_str().unwrap(), "SpogifyMem", "config.json"));
            if mem_path.exists() {
                // if turned on, caches song metadata with file path for use on next launch so don't have to read metadata
                let cache = if self.settings.cache_dir {
                    let mut cache = HashMap::new();
                    self.track_list.iter().for_each(|x| {
                        cache.insert((*x.path).clone(), x.to_cached_song());
                    });
                    cache
                } else {
                    HashMap::new()
                };

                println!("{}", cache.len());

                let save = SpogMem {
                    cache_dir: self.settings.cache_dir,
                    cache,
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

// Returns an index for the next song based on QueueMode
// maybe move to impl SpogApp, but would need another fn for queueing future songs
#[inline(always)]
pub fn queue_track(state: &QueueMode, index: usize, num_songs: usize) -> usize {
    match state {
        Shuffle => get_rand_track(num_songs),
        Next => get_next_track(index, num_songs),
        Loop => index,
    }
}

// Gets a random song
#[inline(always)]
pub fn get_rand_track(num_songs: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..num_songs)
}

// Gets the next song
#[inline(always)]
pub fn get_next_track(index: usize, num_songs: usize) -> usize {
    if index < num_songs {
        index + 1
    } else {
        0
    }
}

// Read metadata and assign file type/extension
pub fn read_metadata(file_path: &String, format: SupportedFormat) -> SongData {
    // First, need extension so we try to read the correct encoding
    // THIS IS GOING TO BE LESS HORRIFYING SOON I'M JUST NOT GREAT WITH ERROR HANDLING OKAY
    let date = if let Ok(meta) = std::fs::metadata(file_path) {
        if let Ok(std_time) =meta.modified() {
            if let Ok(diff) = std_time.elapsed() {
                if let Ok(time_dur) = time::Duration::try_from(diff) {
                    if let Ok(local_now) = OffsetDateTime::now_local() {
                        if let Some(local_file) = local_now.checked_sub(time_dur) {
                            local_file
                        } else {OffsetDateTime::UNIX_EPOCH}
                    } else {OffsetDateTime::UNIX_EPOCH}
                } else {OffsetDateTime::UNIX_EPOCH}
            } else {OffsetDateTime::UNIX_EPOCH}
        } else {OffsetDateTime::UNIX_EPOCH}
    } else {OffsetDateTime::UNIX_EPOCH};

    if let Ok(tagged_file) = lofty::read_from_path(file_path) {
        let duration = tagged_file.properties().duration();
        if let Some(meta) = tagged_file.primary_tag() {
            let title = match meta.title() {
                Some(title) => title.to_string(),
                None => "".to_string(),
            };

            let artist = match meta.artist() {
                Some(artist) => artist.to_string(),
                None => "".to_string(),
            };

            let album = match meta.album() {
                Some(album) => album.to_string(),
                None => "".to_string(),
            };

            SongData {
                title,
                artist,
                album,
                date,
                duration: Some(duration)
            }
        } else {
            SongData::from_duration(duration)
        }
    } else {
        SongData::default()
    }

    // match format {
    //     SupportedFormat::mp3 => {
    //         read_mp3(file_path, dt)
    //     },
    //     SupportedFormat::m4a => {
    //         read_m4a(file_path, dt)
    //     },
    //     _ => {
    //         ("".to_string(),"".to_string(),"".to_string(), OffsetDateTime::UNIX_EPOCH)
    //     }
    // }
}

// fn get_modified(file_path: &str) -> Result<(), dyn Error> {
//     std::fs::metadata(file_path)?.modified()?
// }

// Take a song file path and try to return title, artist, and album
// *** This is for m4a encoding
pub fn read_m4a(dir_path: &str, dt: OffsetDateTime) -> (String, String, String, OffsetDateTime) {
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
            (title, artist, album, dt)
        },
        Err(_) => {
            ("".to_string(), "".to_string(), "".to_string(), dt)
        }
    }
}

// mp3 metadata reading
pub fn read_mp3(dir_path: &str, dt: OffsetDateTime) -> (String, String, String, OffsetDateTime) {
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
            (title, artist, album, dt)
        },
        Err(_) => {
            ("".to_string(), "".to_string(), "".to_string(), dt)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpogMem {
    cache_dir: bool, // Whether the app should try to used cached metadata
    cache: HashMap<String, CachedSong>, // Cached metadata, just empty if not enabled on last exit
    dir_path: String, // Directory last used
    current_index: usize, // Index last used
    volume: f32, // Volume when closed
    queue_mode: QueueMode, // Queue mode when closed ***I THINK doesn't work sometimes
    volume_mode: SliderMode, // Lin or exp volume slider, doesn't work yet so doesn't do anything
    favorites: Vec<usize>, // Favorited songs, will need to make this more robust to work with changing directories
    saved_dirs: Vec<String>, // Saved directories for easy switching
}

impl SpogMem {
    pub fn default() -> Self {
        SpogMem {
            cache_dir: false,
            cache: HashMap::new(),
            dir_path: r"D:\Music\playlist\music".to_string(), 
            current_index: 0, 
            volume: 0.05,
            queue_mode: QueueMode::Next,
            volume_mode: SliderMode::Linear,
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
