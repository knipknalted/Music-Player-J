# Music Player J

A music player I'm making in rust as a personal project. Currently supports MP3, Vorbis, Flac, & AAC encoding.  
All audio playback is handled by rodio  
GUI is with egui  
Other dependencies are in cargo.toml


## Short Term Goals

Option for exponential volume slider, so it is less sensitive at lower values but can still have a wide range  
Adjust legend bar (title...artist...album) layout  
Remove lib.rs, as I now know is unnecessary and ineffecient for a binary project like this  
Improve color shift/interpolation  
Add support for WAV and Opus files
Add support for more metadata encoding (currently only id3 and mp4 containers)
Improve settings window sizing (maybe part of entire settings window overhaul?)


## Long Term Goals

Make settings window nicer  
Improve file format recognition and handling  
Implement better song playback control, i.e. a progress bar for seeking  
Add fancy visuals, ex. an audio visualizer in the background  
Detect song duration, which has been surprising difficult  
Implement alternative ways to play files, like playing a specific file without reading the entire directory  
Tweak the mini-mode ui layout  
Learn more about error handling practices to impliment for unhandled write!() errors in using app memory  
Or just figure out how to use egui built in memory with .ron's and use that  
Detect system sleep so the app can automatically reset the output stream, otherwise it must be reset manually(in settings) or the app restarted  


## Things to monitor

Caching song metadata
Playing a song with arrow keys & enter (froze the app a couple of times when I tried it, have been completely unable to reproduce)


## Accomplished Goals (to keep track of what I've done mostly)

Impliment navigation and playing songs using up/down arrow keys when using search bar (1.3)  
Create actual ui visuals for some placeholders, like mode button and speaker/mute button (1.3)  
Make the search bar caret position accurate and(maybe) impliment navigation with left/right arrow keys in search bar (1.3-1.4)  
Implement sorting track list by title, artist, album, etc.  


## Patch Notes (Here until I have a better place for them)

### 1.3 (first recorded patch notes)

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