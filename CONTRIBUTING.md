# Contributing to RadQC

Thanks for your interest in contributing.

## Prerequisites

- **Rust** ≥ 1.77 via [rustup](https://rustup.rs).
- **Node** ≥ 20 (e.g. via [fnm](https://github.com/Schniz/fnm) or [nvm](https://github.com/nvm-sh/nvm)).
- **Tauri CLI 2.x**: `cargo install tauri-cli --version "^2" --locked`.
- **OS-specific build dependencies**:
  - **Linux**: `sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev libxdo-dev`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Visual Studio Build Tools (with the C++ workload) and the WebView2 Runtime (preinstalled on Windows 10/11 by default).

## Quick start

```bash
git clone https://github.com/Neclow/radqc.git
cd radqc
npm ci
cargo tauri dev
```

The first `cargo tauri dev` compiles the full Tauri runtime — expect 5–15 minutes on a cold cache. Subsequent runs are near-instant (Vite hot-reload for the frontend, incremental Rust compile for the backend).

## Project layout

- `src/` — SvelteKit + TypeScript frontend (Svelte 5, runes-based reactivity).
- `src-tauri/` — Rust backend.
  - `src/lib.rs` — Tauri commands (`list_pngs`, `read_project`, `save_project`) and their unit tests.
  - `tauri.conf.json` — window, identifier, and bundle configuration.
  - `capabilities/` — permissions the webview is allowed to invoke.
- `.github/workflows/` — CI (Linux quick checks on push/PR) and installer-build matrix on push to `main`.

## Tests

```bash
# Rust unit tests (list_pngs, read_project, save_project, round-trip)
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend type and component check (Svelte + TypeScript)
npm run check
```

CI runs both on every push to `main` and on every pull request.

## Building production installers

For your current OS:

```bash
cargo tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`:

- Linux: `.deb` (Debian/Ubuntu) and `.AppImage` (universal).
- macOS: `.dmg`.
- Windows: `.msi` and `.exe`.

For all three operating systems without leaving your machine: push to `main`. The release workflow builds on `ubuntu-latest`, `macos-latest`, and `windows-latest` runners and uploads the resulting installers as workflow artifacts (visible from the Actions tab → run → Artifacts).

## Pull request flow

At least for now, while the project is in early development, the maintainer may push small changes directly to `main`. **Pull requests from contributors are welcomed** and are the preferred way to land outside changes; PR-and-review discipline will tighten as the project matures.

- Fork the repo and create a branch off `main`.
- Keep commits focused — one logical change per commit, so the diff is easy to review.
- Push and open a pull request against `main`.
- CI must be green before review.

## Questions and issues

Please [open an issue](https://github.com/Neclow/radqc/issues) for bug reports, feature requests, or design discussion.
