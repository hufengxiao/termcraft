# TermCraft 🏔️

A voxel world in your terminal. Minecraft, but make it TUI.

**v20.0** — 22 source files, 4112 lines of Rust | 20 versions | 19 commits

## 🎮 Features

| Category | Feature |
|----------|---------|
| **World** | Procedural terrain with 10 biomes (Plains, Desert, Snow, Forest, Mountains, Ocean, Swamp, Jungle, Badlands, Mushroom) |
| **World** | 3D Perlin noise caves, ore veins (Coal/Iron/Gold/Diamond) |
| **World** | Structures: Desert Temples, Witch Huts, Mineshafts |
| **World** | Nether dimension with obsidian portals |
| **Rendering** | DDA raycasting (LUT-optimized, inlined hot path) |
| **Rendering** | ASCII PBR materials (metalness, roughness, reflectance) |
| **Rendering** | Bloom effect, distance fog, sky gradients, day/night cycle |
| **Rendering** | Double-buffered frame diff, zero-alloc bloom |
| **Rendering** | Mini-map HUD (15×15 overhead view) |
| **Physics** | Momentum + friction, gravity, fall damage |
| **Survival** | Health (20 HP), hunger, saturation, starvation |
| **Survival** | Mob damage (Zombie/Skeleton/Spider/Slime) |
| **Survival** | Death + respawn (lose half XP) |
| **Tools** | 14 tools (Wood/Stone/Iron/Diamond × Pickaxe/Axe/Sword/Shovel) |
| **Tools** | Durability, mining speed, correct tool bonuses |
| **Items** | 36-slot inventory, hotbar, block drops |
| **Items** | Food system (Apple, Bread, Pork, Golden Apple) |
| **Crafting** | 3×3 grid with 12+ recipes |
| **Crafting** | Smelting (Ore+Coal→Ingot), tools, torches, furnace |
| **Redstone** | Dust, torch, lever, lamp + BFS signal propagation |
| **Redstone** | 4-bit virtual CPU (8 registers, 16 opcodes) |
| **Mobs** | A* pathfinding, 4 types, biome-specific spawning |
| **Mobs** | Spatial audio, mob drops |
| **Fluids** | Water spreading, gravity flow, source/drain |
| **Audio** | 3D spatial sound (distance + angle panning) |
| **Scripting** | Lua 5.4 engine (mlua), F6 to execute scripts |
| **Networking** | tokio UDP Server/Client scaffolding |
| **Performance** | LUT colors, inline DDA, frame timing HUD |
| **Performance** | Chunk-based 16³ sub-chunks, async generation |

## 🎮 Controls

| Key | Action |
|-----|--------|
| WASD | Move (with momentum) |
| Arrow keys | Look around |
| Mouse Left | Break block |
| Mouse Right | Place block |
| Space | Jump |
| E | Place block |
| Q | Break block |
| R | Eat food |
| 1-9 | Select hotbar slot |
| F5 | Save world |
| F6 | Execute Lua script |
| Esc | Quit |

## 🚀 Run

```powershell
# Requires MSYS2 MinGW in PATH
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cd D:\github\termcraft
cargo run --release
```

## 📐 Architecture

```
src/
├── main.rs       — Entry point (22 modules)
├── biome.rs      — 10 biome types with noise-based selection
├── block.rs      — 23 block types + color/glyph LUTs
├── camera.rs     — DDA renderer, fog, bloom, minimap, PBR
├── cpu.rs        — 4-bit Redstone virtual processor
├── dimension.rs  — Nether dimension + portal mechanics
├── fluid.rs      — Water/lava fluid dynamics
├── game.rs       — Game loop, physics, survival, rendering
├── input.rs      — Keyboard + mouse input
├── item.rs       — Inventory, crafting, food, tools
├── mob.rs        — A* pathfinding mob AI (4 types)
├── network.rs    — tokio UDP multiplayer
├── pbr.rs        — ASCII PBR material system
├── player.rs     — Momentum physics, health, hunger
├── redstone.rs   — BFS signal propagation
├── save.rs       — Binary save/load (serde+bincode)
├── script.rs     — Lua scripting engine
├── sound.rs      — 3D spatial audio (rodio)
├── structure.rs  — Structure generation templates
├── tool.rs       — Tool types, durability, mining speed
├── world.rs      — Chunk-based world generation
└── xp.rs         — Experience + enchanting system
```

## 📊 Version History

| v0.1 | Initial prototype |
|------|-------------------|
| v1.0 | Day/night, caves, villages, save, sound, chunks |
| v2.0 | DDA rendering, frame diff, redstone, inventory, crafting |
| v3.0 | Biome system (6 biomes) |
| v4.0 | Mobs with A* pathfinding, spatial audio |
| v5.0 | Fluid dynamics, Nether dimension, bloom |
| v6.0 | Lua scripting, biome-specific mobs |
| v7.0 | Mini-map HUD |
| v8.0 | LUT colors, inline DDA, zero-alloc bloom, frame timing |
| v9.0 | ASCII PBR materials |
| v10.0 | Redstone CPU (4-bit processor) |
| v11.0 | Mouse controls, survival (health/hunger) |
| v12.0 | Tool system (durability, mining speed) |
| v13.0 | Ore generation (Coal/Iron/Gold/Diamond) |
| v14.0 | Complete crafting (12+ recipes) |
| v15.0 | XP system + enchanting |
| v16.0 | 10 biomes, structures, food |
| v17.0 | Food eating, mob drops |
| v18.0 | Smelting, advanced redstone |
| v19.0 | Multiplayer polish, chat |
| v20.0 | Respawn, death screen, final polish |

## License

MIT
