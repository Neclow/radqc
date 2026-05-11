# RadQC

A standalone desktop application for manual quality-control (QC) annotation of radiology images.

## Getting started

1. Download the installer for your operating system from the [Releases page](https://github.com/Neclow/radqc/releases).
2. Run the installer.
3. Follow the [Tutorial](tutorial.md).

## About

RadQC supports manual QC review of radiology image cohorts by reviewers who do not share the same compute infrastructure as the dataset's owners. The data stays where the user has it; the tool ships as a small native installer that the user points at their own image folder at runtime.

For each image, the reviewer marks a **Minor** or **Major** quality flag (or skips it as acceptable), records a free-text reason, and the annotations are persisted to a YAML file on disk. The application runs entirely locally — no network calls, no cloud storage, no automated analysis.

RadQC currently supports PNG and JPEG images. Support for DICOM or TIFF can be added if there is interest — please [open an issue](https://github.com/Neclow/radqc/issues).

!!! warning "Disclaimer"
    RadQC is a research tool, not a medical device. It performs no automated analysis, makes no clinical interpretations, and is not intended for clinical decision-making. RadQC is shared for research purposes only and is not meant to be used in clinical practice.

## Source code

[github.com/Neclow/radqc](https://github.com/Neclow/radqc) — open source under Apache-2.0.
