Building a basic TUI for Home Assistant in Rust. Serves as a project to learn Rust and it's TUI interface. Heavily inspired by the ytui-music rust app as I try to learn how the pieces fit together. https://github.com/sudipghimire533/ytui-music/


Goals for the next commit:
- [-] Refactor out some repeated code in each module.
    This is not perfect. Some repeat code has been removed but I'm sure as the code becomes more modular & less of a spaghetti code base more will present itself. 
- [ ] Being able to take in User input, then send that state back to HAOS (raw input or maybe do some very basic type matching or something?)
- [X] Comments!
  While the comments aren't amazing. Things are documented so it makes development a smidge easier. 
- [ ] Unit tests!
- [ ] Confirm a close or if there is a "<CTRL> + q" we can force quit. 
- [ ] Pick a license

Some fun next steps:
- [-] I want to be able to create pop ups that allow for me to select a
presented element (where relevant) and then send a post request back
to the HAOS instance.
    So they're created, display, close. But, to reach a neatish MVP, I need to have functionality that lets me send state back to HAOS. 
    That'll come with the next commit. Should be functional at the time of the first upstream push. 
- [ ] Figure out how tabs work, that'd be cool

Long term goals:
- [ ] Implement the HAOS backend using websockets/native app? Would be a good
  opportunity to learn more about that.
- [ ] Benchmarking. How go fast? Maybe I should do this before
  refactoring? I do think reducing repeat code is a good idea tho, so
  maybe this afterward.
- [ ] Redesign the UI. It could be more functional. 
- [ ] I'd really like to implement some level of "vim" style movements? I'll have to research how that is handled. 
        Build some kind of n-ary syntax tree or something? Would make validating commands are valid easy but then going from the "yes, it's valid" to actually executing code might be hard. That, or it's where the magic lies. 
- [ ] Should really add a more rich type system style of things for stuff like lights, sensors, alarms, locks, etc. so that the change state UI is a lot more useful. 
