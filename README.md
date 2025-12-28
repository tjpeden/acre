# ACRE - Ant Colony: Relentless Ecology

A deep simulation game inspired by Dwarf Fortress, featuring leafcutter ants. Manage your colony through pheromone-based influence as ants autonomously forage, farm fungus, and expand their underground empire.

## Concept

You guide a leafcutter ant colony from a single founding queen to a thriving civilization. Unlike traditional RTS games, you don't directly control individual ants. Instead, you influence behavior by placing pheromones—dig here, forage there, avoid this area. Ants respond to these chemical signals while following their own instincts and needs.

**Core Loop:**
```
Surface Plants → Leaf Fragments → Nest → Mulch → Fungus Garden → Food
```

## Features (Planned)

- **Deep Simulation**: Ants with hunger, age, and caste-based behaviors
- **Pheromone Control**: Influence your colony through chemical signals
- **Fungus Farming**: Leafcutter-style agriculture with resource chains
- **3D World**: 64x64x64 tile world with 16 levels above ground and 48 below
- **Emergent Storytelling**: Complex behaviors emerge from simple rules

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys | Pan camera |
| Scroll Wheel | Zoom in/out |
| `[` or `,` | Go down a z-level |
| `]` or `.` | Go up a z-level |
| Space | Pause/Resume |
| 1/2/3 | Set speed (1x/2x/4x) |

## Building & Running

Requires Rust (2024 edition) and Cargo.

```bash
cargo run        # Run the game
cargo build      # Build only
cargo test       # Run tests
cargo clippy     # Lint
cargo fmt        # Format code
```

## Architecture

Built with [Bevy 0.17](https://bevyengine.org/), an ECS game engine.

- **World Grid**: 64³ tile array stored as a resource
- **Entities**: Ants, trees, eggs, food—all dynamic objects
- **Systems**: Autonomous behaviors, pheromone diffusion, rendering

## License

MIT
