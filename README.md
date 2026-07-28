# [Engine Name] 
> A brutal, high-performance, raw Vulkan (ash) toon rendering engine built with Slang and strict Data-Oriented Design.

[Insert your 10-second toon ocean / anime-style shader demo GIF here]

---

## Project Status & Context
**Currently balancing this engine development with preparation for university entrance exams (Mathematics/Physics). Heavy features are frozen until early 2028.**

---

## WARNING: THIS IS NOT A GENERAL-PURPOSE ENGINE.
This engine is built for **ONE** specific purpose: to power my own standalone anime-style game without fighting UE5/Unity.

* **No deep OOP hierarchies.** (Strictly Data-Oriented SoA layouts)
* **No visual node scripting.** (Write Slang code like a real programmer)
* **Zero support** for your generic 2D pixel art or photorealistic architectural visualization.

If you use this code and your PC melts, or your project breaks, read the MIT/Apache 2.0 disclaimer. You have been warned. Enjoy the pure DOD speed.

---

## Tech Stack & Architecture

This architecture rejects high-level wrappers. Everything is controlled at the byte level to maximize cache efficiency and deterministic data flow.

* **Core Pipeline:** Rust (`ash`, `vma`, `winit`, `glam`/`nalgebra`)
* **Shaders:** Slang (utilizing Autodiff `[Differentiable]` and `bwd_diff` for procedural toon math)
* **Asset Pipeline:** Python + FlatBuffers (`flatc`) + `bpy` (Blender custom binary exporter)
* **Editor GUI:** TypeScript + Tauri + React + Tailwind CSS / shadcn/ui
* **Paradigm:** Rigid Data-Oriented Design (SoA memory alignment) & Functional Data Flow

## Quick Start (Instant Demo)

Experience the raw Vulkan + Slang toon renderer immediately. This demo runs on lightweight, pre-compiled FlatBuffers payloads without spinning up the TypeScript editor or Python pipelines:

```bash
git clone https://github.com
cd your-engine
cargo run --example toon_surf
```
*(No Slang compiler, Blender, or Node.js setup required for this minimal showcase.)*

## Deep Architecture (The Deep Ocean)

This engine is built to challenge the absolute deep ocean of AAA-grade runtime architecture from scratch. We don't hide Vulkan's verbosity; we embrace it to control memory.

Detailed architectural decisions, custom FlatBuffers schemas, struct memory paddings, and Slang binding specifications are strictly documented in the `/docs` directory:
* `/docs/adr/` - Architecture Decision Records
* `/docs/arch/` - Structural & Memory Layout Diagrams
* `/docs/requirements/` - Strict layout constraints for CPU-to-GPU data transfer

---

## License

Dual-licensed under either of:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))
