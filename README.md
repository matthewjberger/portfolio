# Portfolio

Personal portfolio at [matthewberger.dev/portfolio](https://matthewberger.dev/portfolio/), rendered live by the [Nightshade](https://github.com/matthewjberger/nightshade) game engine. The engine runs inside a web worker against an OffscreenCanvas through WebGPU, and a [Leptos](https://leptos.dev) UI scrolls over a 3D world: page scroll flies the camera between section vignettes, the pointer adds parallax and nudges floating set pieces, and a hidden physics game ([Nightshade Siege](https://github.com/matthewjberger/nightshade-viewer)) ships as an easter egg.

All portfolio content lives in `data/*.toml` and is embedded at compile time; editing content never touches component code. Browsers without WebGPU get the full content over a CSS backdrop.

## Workspace

- protocol, the message types the page and worker share.
- worker, the wasm module inside the web worker: the world build, the scroll-driven camera tour, ambient animation, pointer floaters, and the Siege game systems.
- the root crate, the Leptos UI: nav, hero, highlights, experience timeline, projects, crates, skills, education, and the game chrome.

## Quickstart

Tooling is pinned in [mise.toml](mise.toml). Install [mise](https://mise.jdx.dev) and [just](https://github.com/casey/just), then:

```bash
just init
just run
```

Serves at http://127.0.0.1:8080. Needs a browser with WebGPU and OffscreenCanvas-in-workers support for the 3D layer (Chromium 113+, Firefox 141+); content renders regardless.

## Deployment

GitHub Actions ([.github/workflows/deploy.yml](.github/workflows/deploy.yml)) builds the worker wasm, the stylesheet, and the release bundle with `--public-url /portfolio/`, then publishes to GitHub Pages at matthewberger.dev/portfolio on every push to main.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
