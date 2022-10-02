STUFF I DID IN CASE OF NEAR FUTURE PROBLEMS
--------------------------------
Switched from using an externally defined function to play a song, to using a method on the app struct
In the method, use a scoped thread
Not cloning the Arc<Sink> anymore, able to use self.sink in scoped thread
Switch Song struct Strings to Rc<Strings>



Goals
--------------------------------
Make settings window nicer, maybe use Area instead of Window
*Maybe* make the search bar cursor blink, and work with arrow keys(big maybe)
*Eventually* add persistent settings for stuff like directory path, etc.



Done, but not tested extensively
--------------------------------
Fix new search bar filter
Try to get the song metadata reading to work so I can at least adjust the rendering of song card with metadata
Adjust song card display, maybe align numbers closer to left and fixed distance
Move the "NOW PLAYING" readout to the right side of the control bar



Done
--------------------------------
Make the whole app window have a minimum size, not sure how to yet

