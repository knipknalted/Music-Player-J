# Music Player J

A music player I'm making in rust as a personal project. Currently only works with m4a/mp4/mp3 encoding.  


## Short Term Goals

Impliment navigation and playing songs using up/down arrow keys when using search bar  
Option for exponential volume slider, so it is less sensitive at lower values but can still have a wide range  
Create actual ui visuals for some placeholders, like mode button and speaker/mute button  
Make the search bar caret position accurate and(maybe) impliment navigation with left/right arrow keys in search bar  
Adjust legend bar (title...artist...album) layout  
Implement sorting track list by title, artist, album, etc.  
Remove lib.rs, as I now know is unnecessary and ineffecient for a binary project like this  


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


## Things to monitor

App memory that persists between launches  
