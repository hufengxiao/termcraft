# TermCraft 🏔️

A voxel world in your terminal. Minecraft, but make it TUI.

## Features

- 🌍 **Procedural terrain** — Simplex noise terrain with hills, plains, coastlines
- 🕳️ **3D caves** — Perlin noise cave systems underground
- 🌳 **Vegetation** — Trees, flowers, tall grass
- 🏘️ **Villages** — Auto-generated wooden houses on plains
- 🌅 **Day/Night cycle** — Dynamic lighting, sky gradients, stars at night
- 🌫️ **Distance fog** — Smooth fog blending to sky color
- 💾 **Save/Load** — Binary world persistence (F5 to save, auto-loads)
- 🔊 **Sound effects** — Procedural audio for steps, place, break
- ⚡ **Chunk system** — 16×16 chunk-based storage for performance

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Arrow keys | Look around |
| Space | Jump |
| E | Place block |
| Q | Break block |
| 1-5 | Select block type |
| F5 | Save world |
| Esc | Quit |

## Run

```bash
# Make sure MSYS2 MinGW is in PATH
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cargo run
```

## Architecture

```
src/
├── main.rs      — Entry point
├── block.rs     — Block types (grass, stone, wood, etc.)
├── world.rs     — World generation, chunk storage, caves, villages
├── camera.rs    — Raycasting renderer, fog, day/night lighting
├── player.rs    — Player state and physics
├── game.rs      — Game loop, input handling, time system
├── input.rs     — Keyboard input mapping
├── save.rs      — Binary save/load with serde + bincode
└── sound.rs     — Procedural audio with rodio
```

## License

MIT
