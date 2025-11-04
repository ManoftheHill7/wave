# Wave

Name is a placeholder. The concept is a survival crafting game, where when the tide is low, you explore a set of caves for resources and treasure. When the tide rises, you need to get out before you get trapped. The core fun of the game, should be navigating procedurally generated caves with a varity of movement options. Jumps, dashes, grappling hooks, gliders, etc.

## Design decisions

- [DECIDED]: The game is a survival crafting game, but mining is not the focus. The focus is on fun platformer movement
- [OPEN]: Should the game be a roguelite? Or should it focus more on base building? How quickly should it be able to be beaten?
- [OPEN]: Should tools have durability? Or once you get a tool, do you have it? Does this include movement tools? Can you repair tools?
- [OPEN]: Farming? How to do this nicely on a side view 2d
- [OPEN]: How should healing work? A food system? A potion system? Heal, when resting? No healing ever?
- [OPEN]: How to indicate to the player the tide will rise? Clock on the hud? Sounds?
- [OPEN]: How to implement water? Static? Cellular automata? Water that rises independent of terrain? Smooth particle hydrodynamics?
- [OPEN]: Should there be boss battles? How common?

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
- [ ] Dynamic Water
- [ ] Tidal cycle
- [ ] Inventory system
- [ ] Player health/HUD
- [ ] Tool - Dash
- [ ] Tool - Pickaxe
- [ ] Tool - Grappling Hook
- [ ] Ore generation
- [ ] Stala(gm/ct)ites
- [ ] Bat enemies
- [ ] Mole enemies
- [ ] Crafting system

### Tier 2 Tasks

Things that I would really like to have

- [ ] Save/load/new game
- [ ] Dynamic lighting
- [ ] Parallax backgrounds
- [ ] Multiple tile layers, allow placing blocks in foreground/background
- [ ] Multiple cave biomes as you get deeper
- [ ] Day night cycle
- [ ] Forageable foods
- [ ] Tool - Fishing rod
- [ ] Fishing. Maybe like Stardew Valley?
- [ ] Dungeons. Sets of connected handmade rooms to find important items
- [ ] Tool - Glider
- [ ] Multiple cave generation types
- [ ] Multiple enemy types
- [ ] Boss battle

### Tier 3 Tasks

Things I will get to if there is time

- [ ] Additional game modes
  - [ ] Hardcore mode. Dead is dead
  - [ ] Relaxed mode. No tidal pressures
  - [ ] Creative mode. Infinite of all resources
- [ ] Decorative tiles for base building

## Links

- https://lodev.org/cgtutor/raycasting.html
- https://www.jgallant.com/2d-liquid-simulator-with-cellular-automaton-in-unity
- https://lucasschuermann.com/writing/implementing-sph-in-2d
- https://nothings.org/gamedev/herringbone/herringbone_tiles.html
