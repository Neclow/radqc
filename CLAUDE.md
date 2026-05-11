# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A **greenfield** standalone build of a manual X-ray QC tool for radiologists. The `starter_kit/`
directory holds **reference material copied from a parent research project** —
`starter_kit/qc_image.py` (the original script) and `starter_kit/qc_tool_spinoff.md` (the
rationale doc). They describe the original implementation and the constraints driving this
spin-off. They are not code to maintain or a spec to implement literally; treat the whole
directory as a starter-kit reference, not as build artifacts.

The short version of why a standalone tool exists at all: external (ETHZ-side) radiologists need
to QC the X-ray dataset, but they cannot get a KU VPN, SIF is data-only, and hosting sensitive
data on a non-secure KU server is a GDPR non-starter. The unlock is to **decouple the tool from
the data** — ship a generic standalone app that the user points at their own manifest CSV + image
directory at runtime. Data residency becomes the user's institution's problem.

## Current state

- `starter_kit/qc_image.py` — reference only. The original parent-project script. Imports from a `src._config`
  module that does not exist here, hard-codes `record_id` / `timepoint` / `site` and a fixed
  timepoint vocabulary (`start`, `end`, `6mo`, `12mo`), and uses a read-modify-write CSV append
  pattern. Useful as a UX/taxonomy reference; do not run, port, or refactor it in place.
- `starter_kit/qc_tool_spinoff.md` — reference only. Design rationale carried over from the parent project.
  Read it for *why* the tool exists and the constraints it has to respect, but the target
  architecture below is the working brief, not the document itself.
- `pixi.toml` is a stub (no tasks, no dependencies declared yet).

## Target architecture (decided 2026-05-11; supersedes the starter-kit doc)

- **Tauri (Rust shell + Svelte/TypeScript frontend).** Not Streamlit, not Python. The
  Python+Streamlit+SQLite plan in `starter_kit/qc_tool_spinoff.md` is superseded — keep that doc
  only for the rationale around data-decoupling, GDPR, and binary size.
- **Inputs at runtime**: reviewer ID, project name, and a folder of PNGs picked via the native
  Tauri file dialog. **No manifest CSV.** Image identity = path relative to the selected folder.
- **CSV output, not SQLite.** Each reviewer writes their own local `{project}_{reviewer}.csv` —
  single-writer per file, so the doc's race-condition argument for SQLite doesn't apply. Current
  draft schema: `path, severity, note`. Open: whether to add a timestamp and whether
  re-annotations overwrite or append.
- **PNG only for v1.** JPEG / TIFF / DICOM are out of scope until a concrete user asks.
- **No bundled data, no PHI assumptions, no project-specific column names.**
- **No auth** — runs locally; OS handles access.
- **Distribution**: `cargo tauri build` produces per-platform installers (`.msi` / `.dmg` /
  `.AppImage` / `.deb`) in the 5–15 MB range. Significantly smaller than the PyInstaller plan in
  the starter-kit doc, which was the original motivation for moving off Python.
- **Open-sourceable** (MIT/Apache); avoid references to internal paths or datasets.

## The "non-reason" to remember

The split is justified by **binary size, distribution polish, and publishability** — not by
"reusability across other projects." Don't design speculative abstractions for hypothetical reuse.

## Reason taxonomy (reference)

Checkbox list per the design doc: penetration / rotation / motion / clipping / lateral-AP-mislabel
/ foreign object / processing artifact / other + free text. Severity: minor / major.

## Naming

- **Local working directory**: `tb-annotator/` (kept for filesystem-organization reasons, not the
  public project name).
- **Project name** (in `Cargo.toml`, `package.json`, public-facing): `radqc`.
- **Tauri bundle identifier**: `io.github.neclow.radqc` (open-source convention via the user's
  GitHub handle).

The mismatch between directory name and project name is intentional — do not "fix" it by
renaming the directory unless explicitly asked.

## Commands

The Tauri scaffold is not yet in place. Expected commands once the scaffold lands:

- `cargo tauri dev` — run the app in development mode with hot reload.
- `cargo tauri build` — produce the per-platform installer.
- Frontend-only dev (rare): `npm run dev` inside the frontend subdirectory.

Toolchain prerequisites (user installs these themselves): rustup (Rust ≥1.77), Node (≥20),
`cargo install create-tauri-app tauri-cli --version "^2"`, and Linux apt packages
(`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libssl-dev`, `librsvg2-dev`,
`libayatana-appindicator3-dev`, `libxdo-dev`, `build-essential`).
