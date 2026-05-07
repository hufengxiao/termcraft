# TermCraft 🏔️

A voxel world in your terminal. Minecraft, but make it TUI.

**v8.0** — 18 source files, ~3000 lines of Rust

## Features

| Feature | Description |
|---------|-------------|
| 🌍 **Procedural terrain** | Simplex noise with 6 biomes (Plains, Desert, Snow, Forest, Mountains, Ocean) |
| 🕳️ **3D caves** | Perlin noise cave systems, depth-dependent density |
| 🌳 **Vegetation** | Biome-aware trees, flowers, tall grass |
| 🏘️ **Villages** | Auto-generated wooden houses on plains |
| 🌅 **Day/Night** | Dynamic lighting, sky gradients, stars at night |
| 🌫️ **Distance fog** | Quadratic fog blending to sky color |
| ✨ **Bloom effect** | Bright blocks glow with zero-allocation rendering |
| ⚡ **DDA raycasting** | LUT-optimized, inlined hot path, AVX-ready |
| 🖥️ **Frame diff** | Double-buffered rendering, only changed pixels |
| 📦 **Chunk system** | 16³ sub-chunks, async on-demand generation |
| 🗺️ **Mini-map** | 15×15 overhead map with terrain coloring |
| 🎮 **Physics** | Momentum + friction, smooth acceleration |
| 🎯 **Crosshair** | Target block detection with HUD display |
| 🔴 **Redstone** | Dust, torch, lever, lamp + BFS signal propagation |
| 🤖 **Mobs** | A* pathfinding, 4 types, biome-specific spawning |
| 💧 **Fluids** | Water spreading, gravity flow, source/drain dynamics |
| 🌐 **Nether** | Obsidian portal → Nether dimension with unique terrain |
| 💾 **Save/Load** | Binary persistence (serde + bincode) |
| 🔊 **3D Audio** | Spatial sound panning by distance & angle (rodio) |
| 🎒 **Inventory** | 36-slot inventory, hotbar, block drops |
| ⚒️ **Crafting** | 3×3 grid with recipes (planks, sticks, pickaxes) |
| 📜 **Lua scripting** | mlua 5.4 integration, F6 to execute scripts |
| 🌐 **Networking** | tokio UDP Server/Client scaffolding |
| ⏱️ **Performance HUD** | Real-time μs/fps display |

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
| F6 | Execute Lua script |
| Esc | Quit |

## Run

```bash
# MSYS2 MinGW required in PATH
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cargo run --release
```

## System Resource Usage Profile

| Metric | Value |
|--------|-------|
| Thread count | ~4 (main + tokio workers) |
| Heap allocations/frame | ~2 (frame buffer + diff buffer) |
| Bloom scratch buffers | Pre-allocated, zero per-frame alloc |
| Color lookups | LUT (20 entries), no match overhead |
| DDA inner loop | `#[inline(always)]`, multiply-only step |
| File handles | 0 persistent (save opens/closes on demand) |

## Architecture

```
src/
├── main.rs       — Entry point
├── biome.rs      — Biome system (temperature/moisture noise)
├── block.rs      — 19 block types + LUT constants
├── camera.rs     — DDA renderer, fog, bloom, minimap, mob overlay
├── dimension.rs  — Nether dimension + portal mechanics
├── fluid.rs      — Water/lava fluid dynamics
├── game.rs       — Game loop, physics, double-buffer I/O, frame timing
├── input.rs      — Keyboard input mapping
├── item.rs       — Inventory + 3×3 crafting grid
├── mob.rs        — A* pathfinding mob AI (4 types)
├── network.rs    — tokio UDP multiplayer protocol
├── player.rs     — Momentum physics
├── redstone.rs   — BFS signal propagation
├── save.rs       — Binary save/load
├── script.rs     — Lua scripting engine (mlua 5.4)
├── sound.rs      — 3D spatial audio engine
└── world.rs      — Chunk-based world + procedural generation
```

## Version History

- **v8.0** — Performance: LUT colors, inline DDA, zero-alloc bloom, frame timing
- **v7.0** — Mini-map HUD, biome-specific mobs
- **v6.0** — Lua scripting engine, mob ecosystem
- **v5.0** — Fluid dynamics, Nether dimension, bloom effect
- **v4.0** — Mobs with A* pathfinding, spatial audio
- **v3.0** — Biome system (6 biomes)
- **v2.0** — DDA rendering, frame diff, redstone, inventory, crafting
- **v1.0** — Day/night, caves, villages, save, sound, chunks
- **v0.1** — Initial prototype

## License

MIT
