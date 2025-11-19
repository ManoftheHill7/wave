# Wave

Name is a placeholder. The concept is a survival crafting game, where when the tide is low, you explore a set of caves for resources and treasure. When the tide rises, you need to get out before you get trapped. The core fun of the game, should be navigating procedurally generated caves with a varity of movement options. Jumps, dashes, grappling hooks, gliders, etc.

## Design decisions

- [DECIDED]: The game is a survival crafting game, but mining is not the focus. The focus is on fun platformer movement
- [DECIDED]: The game is not a roguelike. Basebuilding, and returning to your base is a core gameplay component
- [DECIDED]: Tools have duribility, broken tools can still be used.
- [DECIDED]: Farming? No. Only fishing and forging
- [DECIDED]: Eating food to heal, must be at base
- [DECIDED]: Clock on HUD. Crafted by items. Sounds indicate rising water.
- [OPEN]: How to implement water? Static? Cellular automata? Water that rises independent of terrain? Smooth particle hydrodynamics?
- [DECIDED]: Should there be boss battles? Unique boss per layer. Probably wont get to this

## Plan

Note, many of these tasks can and should be broken up into smaller tasks

### Tier 1 Tasks

Things required for the games core concept

- [x] Infinite worldsize
- [x] Basic player physics (Jump, climb, walk, swim)
- [x] Basic character animations (Jump, climb, walk, swim, idle)
- [x] Basic Tileset
- [x] Raycast for player tools
- [x] Cave generation
- [x] Dynamic Water
- [x] Tidal cycle
- [x] Screen Management System
- [x] Item and equipment system
- [x] Inventory UI
- [x] Player health/HUD
- [x] Tool - Dash
- [x] Player Breath hold
- [x] Tileset variants
- [x] Load map from file
- [x] Stala(gm/ct)ites
- [x] Tool - Pickaxe
- [x] Ore generation
- [x] Map tile loading
- [x] Crafting system
- [ ] Crafting recipes
- [ ] Physics Overhaul
- [x] Crafting only when near crafting table items
- [x] Save/load/new game

### Tier 2 Tasks

Things that I would really like to have

#### Lighting
- [x] Dynamic lighting
- [x] Tool - Lamp
- [x] Torches cast light
- [ ] Colored lighting

#### Additional movement options
- [ ] Tool - Grappling Hook
- [ ] Tool - Glider
- [ ] Boots

#### Food
- [ ] Tool - Fishing rod
- [ ] Forageable foods
- [ ] Fishing. Maybe like Stardew Valley?

#### Artistic improvements
- [ ] Animated tiles (furnace, anvil, torches, eg)
- [ ] Parallax backgrounds
- [ ] Multiple tile layers, allow placing blocks in foreground/background
    - Few opening in tunnels, more in caverns
    - Ore can generate in background layers
    - Maybe replace tideblocks with places water flows in from

#### Audio integration
- [ ] Proximity sound effects for torches
- [ ] Layering music as you go deeper
- [ ] Flooding caverns music intensity
- [ ] Sound effects when things happen
- [ ] Looping sound effects for things like mining
- [ ] Dynamic wooshing, and thuds for falling and impact

#### Misc
- [ ] Improved hitbox/collision handling
- [ ] Tide clears building items
- [ ] Bombs explode
- [ ] Chests store items

#### Combat
- [ ] Bat enemies
- [ ] Mole enemies
- [ ] Tool - Spear
- [ ] Crouching

### Tier 3 Tasks

Things I will get to if there is time

- [ ] Additional game modes
  - [ ] Hardcore mode. Dead is dead
  - [ ] Relaxed mode. No tidal pressures
  - [ ] Creative mode. Infinite of all resources
- [ ] Decorative tiles for base building
- [ ] Message in a bottle hints for gameplay
- [ ] Improve water simulation to support natural waves
- [ ] Dungeons. Sets of connected handmade rooms to find important items
- [ ] Day night cycle
- [ ] Multiple cave biomes as you get deeper
- [ ] Multiple cave generation types
- [ ] Boss battles

## Links

- https://lodev.org/cgtutor/raycasting.html
- https://www.jgallant.com/2d-liquid-simulator-with-cellular-automaton-in-unity
- https://lucasschuermann.com/writing/implementing-sph-in-2d
- https://nothings.org/gamedev/herringbone/herringbone_tiles.html
- https://opengameart.org/users/technodono
