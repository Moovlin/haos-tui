Building a basic TUI for Home Assistant in Rust. Serves as a project to learn Rust and it's TUI interface. Heavily inspired by the ytui-music rust app as I try to learn how the pieces fit together. https://github.com/sudipghimire533/ytui-music/


Goals for the next few commits:
- [-] Refactor out some repeated code in each module.
    This is not perfect. Some repeat code has been removed but I'm sure as the code becomes more modular & less of a spaghetti code base more will present itself. 
- [ ] Unit tests!
    Not doing this for the next public one, it's annoying but just don't need it currently. 
- [ ] Bug squashing. 
- [ ] Fix the low hanging UX fruit. 
    - [ ] Figure out why the UI isn't painting right away
    - [ ] Provide some kind of feedback that a request is being sent
    - [ ] Rather than having users manually type in entities, make it so you can select an entity and then send that entity ID. This will limit (I think) some of the services you can interact with, so perhaps add the option to ente raw JSON as well?? 
    - [ ] Right now I think only the "light" service will work. This is trash and while for me, it's the most useful service, it's not fully featured. 
- [ ] Work through the clippy warnings. 

Completed Goals:
- [X] Being able to take in User input, then send that state back to HAOS (raw input or maybe do some very basic type matching or something?)
    I can take state & services now. This means I can turn lights on & off. The UI is super wonky (and requires some bug fixing to make it work in an expected way) but I can user services. 
    This will absolutely require a refactor in the future as it's pretty hacked together. 
- [X] Comments!
    While the comments aren't amazing. Things are documented so it makes development a smidge easier. 
- [X] Pick a license
    Going with MIT. 
- [X] I want to be able to create pop ups that allow for me to select a presented element (where relevant) and then send a post request back to the HAOS instance.
    I can now create pop ups. I created some traits to make my life a little easier but there is some refactoring to be done with all of those elements. 


Some fun next steps:
- [ ] Figure out how tabs work, that'd be cool

Not hard goals but not part of MVP:
- [ ] Confirm a close or if there is a "<CTRL> + q" we can force quit. 

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


Known issues & limitations:
- The way in which I'm handling state is far from optimal. Can end up in situations where, rather than blocking, I instead just go "eh, I'll update the state later". This isn't ideal and can cause problems
- There are a number of gross, quick & dirty unwraps/panics that occur in recoverable situations. These are actually totally useless and should be repaired.
- I basically do nothing with the JSON response from state changes. This means we don't know what the new state is until we hit the next fetch. 
- We will happily send an empty string. There is no input checking and no santizing done. THIS IS A GIGANTIC PROBLEM WHICH MUST BE FIXED (at some point....)
- The requests which we send when there is state prevent
- If you close out of a popup to quickly, it will prevent the request from being sent to HAOS instance. Obviously a giant problem as well at it forces the user to wait. 
