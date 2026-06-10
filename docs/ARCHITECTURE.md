# Architecture

How the portfolio is wired: the crates, the thread split, the message protocol, the content pipeline, and the scroll-driven 3D layer. Paths are relative to the repository root.

## Two threads

The engine runs on a web worker. The page runs on the main thread. They share nothing but messages. The worker owns the OffscreenCanvas, the Nightshade World, and the skybox fetch. The page owns the DOM. It renders the portfolio sections, forwards scroll, pointer, and game input, and updates its signals from the messages the worker sends back.

```
MAIN THREAD (Leptos)                  WEB WORKER (Nightshade)
src/app.rs        compose             worker/src/lib.rs     message pump, render loop
src/bridge.rs     postMessage         worker/src/state.rs   Portfolio, the State impl
src/components/*  sections, viewport  worker/src/ecs.rs     PortfolioWorld (freecs)
src/content.rs    embedded TOML       worker/src/systems/*  tour, ambient, floaters, game
src/state.rs      grouped signals
src/gamepad.rs    controller polling

  transfer_control_to_offscreen()      ->  create_wgpu_renderer(OffscreenCanvas)
  ClientMessage (scroll, glance, ...)  ->  handle_message, systems
  WorkerMessage                        <-  Ready, Stats, Game, GameHit
```

## Crates

protocol holds the message and data types, the one place the wire is defined. worker is the wasm module: the engine World plus a PortfolioWorld, a freecs world whose resources hold the portfolio's own state (the camera tour, ambient animation, the floaters, the skybox inbox, the siege game), driven by free functions in worker/src/systems. The root crate is the Leptos UI: nav, the seven sections, the viewport canvas, the game chrome, and the gamepad loop.

## Content

Every portfolio fact lives in `data/*.toml`: about, experience, education, highlights, crates, projects, skills. src/content.rs embeds the seven files with `include_str!`, parses them once at startup, and leaks the result so every component borrows a `&'static Content`. Editing content never touches component code, and a typo fails the page at startup rather than rendering wrong.

The TOML holds the data, not everything on screen. Section headings and other UI copy live in the components, and the avatar, resume, and highlight images are static files under `static/` referenced by path. The worker embeds its own binary assets with `include_bytes!`: the hero helmet model and the prototype grid texture.

## The message protocol

protocol/src/lib.rs is the contract. The page sends ClientMessage: Init with the transferred canvas, Resize, Scroll progress in 0..1, Glance (the normalized pointer for parallax), PointerMove, PointerButton, Wheel, Touch, Orbit (gamepad camera deltas), SetReducedMotion, and Game commands. The worker sends WorkerMessage: Ready, Stats with the fps, Game with the scoreboard, and GameHit for the floating score popups. Pixel quantities are physical surface pixels, CSS pixels times the device pixel ratio capped at 2.

Messages travel as `{ message }` envelopes through serde_wasm_bindgen. Init also carries the OffscreenCanvas under its own envelope key in the transfer list. runtime/worker.js bootstraps the worker: it buffers any message that arrives before the wasm module finishes initializing, then replays the backlog once the Rust `onmessage` handler is installed, so an early Init is never dropped.

## The camera tour

The page streams its scroll progress to the worker, and worker/src/systems/tour.rs chases it through six keyframes, one per section vignette: focus, yaw, pitch, and radius for a pan-orbit camera. Smoothstep interpolation between keyframes plus the controller's own smoothing turns scrolling into easing. The pointer glance adds a small yaw and pitch offset for parallax, and worker/src/systems/floaters.rs builds a world ray from the same glance to shove the hero orbs, which spring back home with plain integration, no physics engine involved. A prefers-reduced-motion match disables the glance and damps the ambient animation. The tour idles while the siege game owns the camera.

On the page side, a scroll spy tracks which section fills the viewport for the nav highlight and the one-way reveal animations (src/app.rs). Without WebGPU the canvas never mounts and a CSS backdrop sits behind the same sections.

## The world

worker/src/systems/setup.rs prepares the scene once: render settings, a shadow-casting sun, the camera, and the section vignettes built by worker/src/systems/world/level.rs. Each vignette sits at the depth its keyframe focuses on: the hero island with the spinning helmet and the pointer-reactive orbs, the highlights shrine, the experience canyon, the orbiting crate belt, and the finale star field. Ambient motion is declarative: spawn code registers Spinner, Bobber, and Orbiter entries and worker/src/systems/ambient.rs advances them from accumulated time.

The sky starts as the procedural atmosphere. worker/src/systems/sky.rs fetches a pinned Polyhaven HDRI with the engine's ehttp re-export, the callback writes the bytes into an inbox behind an Arc<Mutex>, and a poll system swaps in the HDR skybox once they arrive.

## The siege game

A hidden physics game ships as an easter egg. Starting a level enables physics, builds the arena far from the tour vignettes, and stacks the level layout (worker/src/systems/game.rs). A tap fires an emissive cannonball along the picking ray through the tap position. Knocking a glowing target off its perch scores points with a combo window, clearing every target wins with a bonus per unused shot, and running out of shots fails after a settle delay. The worker posts Game status whenever it changes and GameHit per knockout, and the page renders the scoreboard, popups, and overlays in src/components/game.rs.

Levels can open with a cutscene: a letterboxed camera sweep on a separate cinematic camera that ends exactly on the player camera's aim pose, so the handoff is seamless. The page persists progress in localStorage: the highest unlocked level, the best score per level, and which intros have played (src/state.rs).

src/gamepad.rs polls the first connected controller every frame and routes it by mode: the portfolio scrolls and moves DOM focus, overlays cycle their buttons, the intro skips, and gameplay orbits, zooms, and fires at the screen center.

## Build

just run builds the worker to wasm with wasm-bindgen and wasm-opt, generates the Tailwind stylesheet, and serves the bundle with Trunk at 127.0.0.1:8080. just check and just lint run cargo check and clippy against the wasm32 target. A push to main runs the same build in .github/workflows/deploy.yml and publishes dist/ to GitHub Pages at matthewberger.dev/portfolio.
