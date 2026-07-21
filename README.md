# Fluid Simulation

A 2D Smoothed Particle Hydrodynamics (SPH) fluid simulation written in Rust with [Bevy](https://bevyengine.org/). 

I built this CPU version as a prototype to nail down the math, spatial hashing, and memory layouts before porting everything over to **WGSL / WebGPU compute shaders**.

## What's implemented so far
* **SPH Physics Pass:** Density estimation, near/far pressure forces, and viscosity.
* **Spatial Hashing:** $O(N)$ neighborhood lookups using grid cell hashing and sorted offset tables.
* **Multi-threaded CPU Pipeline:** Using [Rayon](https://crates.io/crates/rayon) for fast parallel iterations (~150 FPS @ 2,000 particles).
* **Speed Gradient:** Dynamic particle coloring based on velocity using a 256-entry lookup table.
* **Live Inspector:** Real-time parameter tweaking via [`bevy_inspector_egui`](https://crates.io/crates/bevy-inspector-egui).

## How to Run

```bash
cargo run --release
```

### Compilation Note

Build times for `bevy` in default release mode can take a long time on the initial build depending on your hardware.

To speed up local development iteration cycles, use **dynamic linking** in dev mode:

```bash
cargo run --features bevy/dynamic_linking
```

For more details on optimizing build times, check out the [Bevy Fast Compiles Setup Guide](https://bevy.org/learn/quick-start/getting-started/setup/#enable-fast-compiles-optional).

---

## TODO & Future Roadmap

* [ ] **Phase 1 (GPU Port):** Translate SPH compute passes (`run_spatial_hash`, `update_density`, `apply_pressure`) to WGSL compute shaders.
* [ ] **Phase 2 (Parallel GPU Sort):** Implement GPU counting sort using Parallel Prefix Sum (Blelloch scan algorithm).
* [ ] **Phase 3 (Optimization & Benchmarks):** Measure CPU vs GPU performance metrics at scale ($N > 50,000$ particles).
* [ ] **Phase 4 (Far Future):** Port the SPH pipeline to 3D with depth sorting and screen-space fluid rendering.

---

## References & Inspiration

* [Sebastian Lague's Fluid Simulation Series](https://github.com/SebLague/Fluid-Sim) - Primary reference for SPH formulas and spatial hashing architecture.
* [Particle-Based Fluid Simulation for Interactive Applications](https://matthias-research.github.io/pages/publications/sca03.pdf) (Müller et al., 2003) - Original SPH paper for real-time graphics.
* [Smoothed Particle Hydrodynamics Techniques for the Physics Based Simulation of Fluids and Solids](https://sph-tutorial.physics-simulation.org/pdf/SPH_Tutorial.pdf) - Check the [Github](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH)

