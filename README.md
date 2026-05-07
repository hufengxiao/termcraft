# TermCraft 🏔️

A voxel world in your terminal. Minecraft, but make it TUI.

**v4.0** — 15 source files, ~6500 lines of Rust

## Features

| Feature | Description |
|---------|-------------|
| 🌍 **Procedural terrain** | Simplex noise with 6 biomes (Plains, Desert, Snow, Forest, Mountains, Ocean) |
| 🕳️ **3D caves** | Perlin noise cave systems, depth-dependent density |
| 🌳 **Vegetation** | Biome-aware trees, flowers, tall grass |
| 🏘️ **Villages** | Auto-generated wooden houses on plains |
| 🌅 **Day/Night** | Dynamic lighting, sky gradients, stars at night |
| 🌫️ **Distance fog** | Quadratic fog blending to sky color |
| ⚡ **DDA raycasting** | 10x faster than brute-force stepping |
| 🖥️ **Frame diff** | Double-buffered rendering, only changed pixels |
| 📦 **Chunk system** | 16³ sub-chunks, async on-demand generation |
| 🎮 **Physics** | Momentum + friction, smooth acceleration |
| 🎯 **Crosshair** | Target block detection with HUD display |
| 🔴 **Redstone** | Dust, torch, lever, lamp + BFS signal propagation |
| 🤖 **Mobs** | A* pathfinding Zombie & Slime with spatial audio |
| 💾 **Save/Load** | Binary persistence (serde + bincode) |
| 🔊 **3D Audio** | Spatial sound panning by distance & angle (rodio) |
| 🎒 **Inventory** | 36-slot inventory, hotbar, block drops |
| ⚒️ **Crafting** | 3×3 grid with recipes (planks, sticks, pickaxes) |
| 🌐 **Networking** | tokio UDP Server/Client scaffolding |

## Controls

| Key | Action |
|-----|--------|
| WASD | Move (with momentum) |
| Arrow keys | Look around |
| Space | Jump |
| E | Place block (uses inventory) |
| Q | Break block (adds to inventory) |
| 1-9 | Select hotbar slot |
| F5 | Save world |
| Esc | Quit |

## Run

```bash
# MSYS2 MinGW required in PATH
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cargo run
```

## Architecture

```
src/
├── main.rs      — Entry point
├── biome.rs     — Biome system (temperature/moisture noise)
├── block.rs     — 14 block types + redstone components
├── camera.rs    — DDA renderer, fog, lighting, mob overlay
├── game.rs      — Game loop, physics, double-buffer I/O
├── input.rs     — Keyboard input mapping
├── item.rs      — Inventory + 3×3 crafting grid
├── mob.rs       — A* pathfinding mob AI
├── network.rs   — tokio UDP multiplayer protocol
├── player.rs    — Momentum physics
├── redstone.rs  — BFS signal propagation
├── save.rs      — Binary save/load
├── sound.rs     — 3D spatial audio engine
└── world.rs     — Chunk-based world + procedural generation
```

## Version History

- **v4.0** — Mobs with A* pathfinding, spatial audio, noise cache optimization
- **v3.0** — Biome system (6 biomes), biome-aware generation
- **v2.0** — DDA rendering, frame diff, redstone, inventory, crafting
- **v1.0** — Day/night, caves, villages, save, sound, chunks
- **v0.1** — Initial prototype

## License

MIT
