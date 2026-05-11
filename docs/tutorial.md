# Tutorial

This page walks through a complete RadQC session: installing, launching, configuring a project, annotating images, and finding the output file.

## Launching

After installing RadQC, launch the application. You are presented with two cards on the landing screen.

![Landing screen with the two starting cards](images/01-landing.png)

## Create a new project

1. Click **Get started** on the "Create a new project" card.
2. Enter a **reviewer ID** (your initials or any identifier) and a **project name**.
3. Click **Select folder…** under "Image folder" and choose the directory containing the images to review (PNG and JPEG are supported). Subdirectories are walked recursively.
4. Optionally pick a different **output folder** under "Output folder" (defaults to the image folder).
5. Click **Start annotating**.

![Setup form with reviewer ID, project name, and folder pickers](images/02-setup.png)

## Open an existing project

To continue an earlier session:

1. Click **Open project…** on the "Open existing project" card.
2. Select the `.radqc.yaml` project file from a previous session.
3. The application loads the existing annotations and resumes where you left off.

## Annotating an image

For each visible image:

- Pick a flag: **Minor** (usable but with a noted quality issue) or **Major** (unsuitable for use).
- If a flag is selected, provide a short **Reason** describing the issue.
- Leave both fields empty to skip the image.

![Annotation view with the flag and reason controls](images/03-annotation.png)

Click **Save N annotations** to write the page's annotations to the YAML file, then advance to the next page.

## Additional controls

- **Grid size** — view 1, 2, 4, or 8 images per page.
- **Show in original size** — display the image at its intrinsic pixel size (single-image mode only).
- **Filter** — All / Flagged / Unflagged.
- **Path substring search** — narrow the visible image list.
- **Page jump** — type a page number to navigate.
- **Saved panel** — table of all saved annotations; clicking a row jumps to that image.

![Annotation view with the grid set to four images](images/04-grid.png)

## The output file

Annotations are written to a single YAML file at `{output_folder}/{project}_{reviewer}.radqc.yaml`:

```yaml
radqc: 0.1.0
reviewer: neil
project: default
image_dir: /path/to/your/images
annotations:
  patient_001.png:
    severity: minor
    reason: slight rotation
  patient_007.png:
    severity: major
    reason: severe motion blur
```

Each save rewrites the file atomically (temp file + rename) so an interrupted save cannot corrupt the data. Re-annotating an image overwrites its previous entry; no history is retained.

!!! tip "Sharing or analysing the output"
    The YAML file is plain text and self-describing. It can be opened in any text editor, parsed by any YAML library (`pyyaml` in Python, `serde_yaml` in Rust, `js-yaml` in JavaScript, etc.), or shared with collaborators alongside the image folder.
