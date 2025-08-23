# TEXAFORM

## playthrough
- [x] smelter
- [x] solar
- [x] battery 
- [x] lazer cutter
- [x] build solar
- [ ] dog
- [ ] victory 
- [o] extra credit
  - [ ] assembler 
  - [ ] accumulator
    - [ ] build accumulator
  - [ ] run 5 dogs at once

## python TODO
- [ ] deterministic random seed for reproducable testing
- [ ] python implementation (bug reproducability) (later)
- [ ] bug: dog gets stuck in this shape
  - see `bug_dog_stuck` save file
```
 x
x x
x >x
  x
```

## playthrough missing features
- [ ] something fun when victory researched
  - [x] show victory screen or popup with run stats
    - [x] add command counter
    - [x] add random seed value
  - [x] implemented
  - [ ] tested
- [ ] double click agent list centers surface on the agent
  - agents list should be implemented like the other lists

## bugs
- [ ] focus on entity > dog picks entity > focus shoud ???
- [ ] load game starts at top left (0, 0)
- [ ] why is there a delay connecting to agents?

## low priority
- [ ] update ratatui version and migrate from `buf.get_mut()` to `buf.get_cell()`
- [ ] should widgets store a reference to its associated area (Rect)?
- [ ] PhantomType for GridPosition and FramePosition
- [ ] use TextList for agent list
- [ ] implement offset for TextList (when not all items can fit in UI)
- [ ] surface generation should garentee minimum amount of each resource close to starting area
- [ ] clean up UI
  - [ ] when agent selected it is displayed twice (agent log and info)
- [ ] add exit to pause menu
- [ ] add continue to main menu
- [ ] fabricator's BULD should check for room im `buffer_out`
- [ ] moving average window for power graph
- [ ] agents should have methods to help
  - [ ] "set status" method display text in agent's info
  - [ ] "set color" change bg color of agent
  - [ ] "set name" give the dog a name
    - will this confilct with other features like focus?
- [ ] render entities name in the UI with line() instead of kind() everywhere
  - [ ] documentation
  - [ ] agent list
- [ ] input/surface on click and drag select all entites in square and summarize / list them out in info section
- [ ] async game saving / in the background

## Future
- [ ] set up a benchmark to test/understand how long a single frame draw takes (on each Screen)
- [ ] nail down render rate / tick rate / annimation rate
  - [ ] can we have a speed setting (i.e. time between commands accepted by agent)?
- [ ] fog of war
- [ ] secrets
- [ ] bio
- [ ] modified (hightlighted, bold, etc.) documentation rendering 
- [ ] music
- [ ] sound effects 
- [ ] animations
- [ ] CHEAT interface for fun, debugging, and creating test cases
  - [ ] testing framework

## high level idea
* you remote control various robots operating on a remote planetary body
* goal is to teaform and bring life to the planet
* mix of factorio, advent of code, and exapunks
* the time and power consumption of robots should incentivize efficent robots

* do we need good diagnostics, logs, and playback to make it an enjoyable game?


### later?

- [ ] tick rate
  - [x] playtime tracker
  - [ ] effects should be update on render not on game logic tick

- [ ] warn dont abort if port is in use

- [ ] add lore 
  - [ ] transition screen after selecting New Game

- [ ] ability to recycle
  - [ ] lazer cutter outputs scrap
  - [ ] smelter can smelt scrap

- [ ] updating order
  - look into this again, do we really want/need it at this point?
  - how they happen now:
    1. `Event::AgentCommand` sent by `tcp::handle_socket` handled in main loop
  - need to think through main loop
    - what would it look like to only update agents on Event::Tick?
    - how to buffer tcp message events and then sort and resolve them per tick?
    - test out with very large `tick_update_mills`


- [ ] open PR documenting panic of https://docs.rs/ratatui/latest/src/ratatui/buffer/buffer.rs.html#99-109
  - [ ] what are the implications of `debug_assert!` in `index_of`? What happens in --release when bad index?


* create pixle animations via blender/game engine
  * e.g. https://deep-fold.itch.io/space-background-generator https://github.com/Deep-Fold


.........................
.........................
.........xxxxx...........
.......xxxx.xxxx.........
......xxxxx.xxxxx........
.....xxxxxx.xxxxxx.......
.....x...........x.......
.....xxxxxx.xxxxxx.......
......xxxxx.xxxxx........
.......xxxx.xxxx.........
.........xxxxx...........
.........................
.........................

.........................
.........................
.........OOOOO...........
.......OOOO OOOO.........
......OO OO OO OO........
.....OOOO O O OOOO.......
.....OOOOO   OOOOO.......
.....OOOO O O OOOO.......
......OO OO OO OO........
.......OOOO OOOO.........
........OOOOOOO..........
.........................
.........................

.........................
.........................
.......OOOOOOOOO.........
......O  OOOOO  O........
.....OOO  OOO  OOO.......
....OOOOO     OOOOO......
...OO             OO.....
....OOOOO     OOOOO......
.....OOO  OOO  OOO.......
......O  OOOOO  O........
.......OOOOOOOOO.........
.........................
.........................

.........................
.........................
.......OOOOOOOOO.........
.....OO  OOOOO  OO.......
....OOOO  OOO  OOOO......
....OOOOO     OOOOO......
...OO             OO.....
....OOOOO     OOOOO......
....OOOO  OOO  OOOO......
.....OO  OOOOO  OO.......
.......OOOOOOOOO.........
.........................
.........................
