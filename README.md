# RadQC

A standalone desktop application for manual quality-control (QC) annotation of radiology images.

**Status**: v0.1.0 (Beta)

## Quick overview

RadQC supports manual QC review of radiology image cohorts by reviewers who do not share the same compute infrastructure as the dataset's owners. The data stays where the user has it; the tool ships as a small native installer that the user points at their own image folder at runtime.

For each image, the reviewer marks a **Minor** or **Major** quality flag (or skips it as acceptable), records a free-text reason, and the annotations are persisted to a YAML file on disk. The application runs entirely locally — no network calls, no cloud storage, no automated analysis.

RadQC currently supports PNG and JPEG images. Support for DICOM or TIFF can be added if there is interest — please [open an issue](https://github.com/Neclow/radqc/issues).

> RadQC is a research tool, not a medical device. It performs no automated analysis, makes no clinical interpretations, and is not intended for clinical decision-making. RadQC is shared for research purposes only and is not meant to be used in clinical practice.

## Installation

Download the installer for your operating system from the [Releases page](https://github.com/Neclow/radqc/releases) and run it.

The installers are not yet code-signed, so your operating system may warn you the first time you launch RadQC:

- **macOS** — "RadQC cannot be opened because it is from an unidentified developer." Right-click the app in Finder, choose *Open*, then confirm.
- **Windows** — Microsoft Defender SmartScreen may say "Windows protected your PC." Click *More info → Run anyway*.
- **Linux** — `.AppImage` files may need to be marked executable (`chmod +x RadQC*.AppImage`) before they run.

Code-signed releases are a planned improvement.

## Basic usage

See the [tutorial](https://neclow.github.io/radqc/tutorial/) for a step-by-step walkthrough.

## How to contribute

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0. See [LICENSE](LICENSE).

© 2026 Neil Scheidwasser.
