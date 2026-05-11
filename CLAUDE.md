# RadQC

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

RadQC is a standalone manual quality-control (QC) annotation tool for radiology images, built as a Tauri 2 app (Rust shell + SvelteKit/TypeScript frontend) producing native per-platform installers.

The application is decoupled from the data: the user picks an image folder at runtime. No bundled data, no PHI assumptions, no project-specific schema. Annotations are persisted to a YAML file on disk and never leave the user's machine.

## Main objectives

1. **Tauri (Rust + SvelteKit/TS)**. Tiny binaries (~5–15 MB installers), native feel, cross-platform, double-click distribution.
2. **Runtime-decoupled inputs**: reviewer ID, project name, image folder (PNG and JPEG for v1), output folder picked via native dialogs. No manifest CSV.
3. **Per-image annotation**: Minor / Major flag + free-text Reason. Skip-able. Image identity = path relative to the image folder.
4. **YAML output** at `{output_folder}/{project}_{reviewer}.radqc.yaml` with top-level `radqc: <version>` marker, plus `reviewer`, `project`, `image_dir`, and `annotations` (map keyed by relative path). Latest-wins on re-annotation; no history kept; atomic writes (temp file + rename).
5. **Open-source** under Apache-2.0.

## Naming convention

- `radqc` (lowercase) — package name in `Cargo.toml`, `package.json`, GitHub repo URL.
- `RadQC` (mixed case) — display name in window title, headings, app metadata.
- `io.github.neclow.radqc` — Tauri bundle identifier.

Don't globally `s/radqc/RadQC/` — the lowercase form is load-bearing in package metadata.

## Commands

- `cargo tauri dev` — run the app in development mode with hot reload.
- `cargo tauri build` — produce the per-platform installer.
- Frontend-only dev (rare): `npm run dev` inside the frontend subdirectory.

Toolchain prerequisites (user installs these themselves): rustup (Rust ≥1.77), Node (≥20), `cargo install create-tauri-app tauri-cli --version "^2"`, and Linux apt packages (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libssl-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`, `libxdo-dev`, `build-essential`).

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.
