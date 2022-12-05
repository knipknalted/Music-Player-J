# Music Player J

A music player I'm making in rust as a personal project. Currently supports MP3, Vorbis, Opus, Flac, WAV, & AAC encoding.  
All audio playback is handled by rodio  
GUI is with egui  
Other dependencies are in cargo.toml

## Next Patch Goals

Impliment a fixed 10/15s skip forward/backward  
Detect song duration, which has been surprising difficult  
Implement better song playback control, i.e. a progress bar for seeking  
Polish UI with color shifting on  
Add song duration in ui displays & option to sort by duration  


## 0.2.0 Goals

Fully featured song progress bar with seeking  
Completely polished UI for everything implimented thus far  
Complete error handling (app should never crash for any reason)  
Custom/overhauled settings menu that should look and feel better    
Automatically handle audio stream reset on system sleep (do not have to restart app after system sleep)


## Short Term Goals

Adjust legend bar (title...artist...album) layout  
Add support for more metadata encoding (currently only id3 and mp4 containers)
Detect song duration, which has been surprising difficult  
Store output device in memory  


## Long Term Goals

Make settings window format nicer and sizing better adapt to window size  
Add fancy visuals, ex. an audio visualizer in the background  
Implement alternative ways to play files, like playing a specific file without reading the entire directory  
Tweak the mini-mode ui layout  
Learn more about error handling practices to impliment for unhandled write!() errors in using app memory  
Or just figure out how to use egui built in memory with .ron's and use that  
Detect system sleep so the app can automatically reset the output stream, otherwise it must be reset manually(in settings) or the app restarted  
Properly error handle all of the unwraps being called  


## Things to monitor

Doing all song metadata/codec specifics with lofty (crate)  
Caching song metadata  
Playing a song with arrow keys & enter (froze the app a couple of times when I tried it, have been completely unable to reproduce)  
Had the 0.1.3 build crash once in use, first guess is an edge-case indexing out of bounds with something  


## Patch Notes (Here until I have a better place for them)

### 0.1.4
- Added semi-polished button to switch volume slider mode  
- Added support for selecting audio output device  
- Fixed issue with clicking on the currently playing song display not scrolling to it in the list  
- Fixed issue preventing the 'go back' button from restarting the current song if the app's song history was empty  
- *Maybe* fixed a bug very occasionally causing the 'current song display' to be incorrect  

### 0.1.3 (first recorded patch notes)

- Song fading in/out on play/pause  
- Improve search bar rendering accuracy (caret positioning is accurate for large strings)  
- Improve search bar rendering functionality (navigate with left/right arrow keys in search text)  
    - Search bar does nor render properly if text exceeds width of search bar, being worked on I just wanted a break from it
- New loading visual indicator  
- Somewhat working ability to select a song with up/down arrow keys while the search bar is active, and play it with enter  
- Sorting track list by column  
- Temporary context menu (right click) on the current song display  
    - can go out of the window, will eventually need to build my own popup menu to fix that
- (limited) Adjustable playback speed, mostly for debugging  