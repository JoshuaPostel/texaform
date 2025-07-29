# TEXAFORM

## FINISH BY END OF JULY
submit to: https://github.com/ratatui/ratatui/discussions/1886

## Priority TODO
- [ ] finish playthrough
  - change start to:
    - [x] starting with dog
    - [x] starting Focus is dog
- [ ] polish polish polish
- [x] make demo gif with https://github.com/charmbracelet/vhs
- [x] try adding a second DOG at start
- [x] update tutorial
  - [x] place tutorial at top of screen
  - [x] PICK char or name
  - [x] move surface: arrows, page up/down, end/start
- [x] update documentation
  - [x] PICK char or name
  - [x] fab MAKE
  - [x] dog BULD
- [x] lazer -> laser
- [x] max size on comms log
- [x] copy to clipboard functionality
- [x] make PICK/LOAD/etc work with both character or screaming case name
- [x] bug: dog DROP err already occupied destroys current payload

## milestone
- [ ] do a playthrough
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
- [ ] record bugs
- [ ] document everything
- [x] tweak energy levels

## python TODO
- [x] handle goto that is occupied
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
    - [ ] add random seed value
  - [x] implemented
  - [ ] tested
- [ ] random seed for deterministic
  - [ ] map generation
  - [ ] python implementation (bug reproducability) (later)
- [ ] version field in save file
  - how to parse just this?
- [ ] double click agent list centers surface on the agent
  - agents list should be implemented like the other lists

## bugs
- [ ] load game screen: memory allocation of ... bytes failed
  * loading too much/not clearning memory?
- [ ] focus on entity > dog picks entity > focus shoud `INTERNAL_DEV`
- [ ] load game starts at top left (0, 0)
- [ ] why is there a delay connecting to agents?

## low priority
- [ ] cargo: move from package/library to application/binary
- [ ] keybord navigation for everything
  - [ ] add [R] (or [T]?) for technology shortcut
  - [ ] controls pause menu item?
  - [ ] tab based "focus" between areas (highlight focused area)?
- [ ] surface generation should garentee minimum amount of each resource close to starting area
- [ ] clean up UI
  - [ ] when agent selected it is displayed twice (agent log and info)
- [ ] CHEAT interface for fun, debugging, and creating test cases
- [ ] add exit to pause menu
- [ ] add continue to main menu
- [ ] fabricator's BULD should check for room im `buffer_out`
- [ ] moving average window for power graph
- [ ] agents should have methods to help
  - [ ] "set status" method display text in agent's info
  - [ ] "set color" change bg color of agent
  - [ ] "set name" give the dog a name
    - will this confilct with other features like focus?
- [ ] internal code: rename Properties to Entities
- [ ] render entities name in the UI with line() instead of kind() everywhere
  - [ ] documentation
  - [ ] agent list
- [ ] implement uparrow/downarrow for manual command ui
- [ ] input/surface on click and drag select all entites in square and summarize / list them out in info section
- [ ] async game saving / in the background
- [ ] remove either `color_eyre` or `anyhow`

## Future
- [ ] fog of war
- [ ] secrets
- [ ] bio
- [ ] modified (hightlighted, bold, etc.) documentation rendering 
- [ ] music
- [ ] sound effects 
- [ ] animations

## high level idea
* you remote control various robots operating on a remote planetary body
* goal is to teaform and bring life to the planet
* mix of factorio, advent of code, and exapunks

## Idea
* the time and power consumption of robots should incentivize efficent robots
* tradeoff between robot capability and price
* need good diagnostics, logs, and playback to make it an enjoyable game


### later?

- [ ] make info section resizeable

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


### tasks:
- [ ] think through tick/update time
  * discrete step based
    * how small of a timestep?
      * 60 steps a second like factorio?
      * 1 setp a second?


- [ ] open PR documenting panic of https://docs.rs/ratatui/latest/src/ratatui/buffer/buffer.rs.html#99-109
  - [ ] what are the implications of `debug_assert!` in `index_of`? What happens in --release when bad index?

- [ ] update ratatui version and migrate from `buf.get_mut()` to `buf.get_cell()`

- [ ] implement collapsed borders: https://ratatui.rs/recipes/layout/collapse-borders/

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
