<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { open, confirm } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { sanitizeSegment } from "$lib/sanitize";

  type Severity = "" | "minor" | "major";
  type FormState = { severity: Severity; reason: string };
  type GridSize = 1 | 2 | 4 | 8;
  type ThemeMode = "auto" | "light" | "dark";
  type Filter = "all" | "flagged" | "unflagged";

  const VERSION = "0.1.0";
  const REPO_URL = "https://github.com/Neclow/radqc";
  const ISSUES_URL = "https://github.com/Neclow/radqc/issues";

  let reviewer = $state("");
  let project = $state("default");
  let folder = $state<string | null>(null);
  let projectPath = $state<string | null>(null);
  // raw because images and saved are reassigned in bulk, never mutated in place
  let images = $state.raw<string[]>([]);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  let started = $state(false);
  let mode = $state<"choose" | "new">("choose");

  let gridSize = $state<GridSize>(1);
  let currentPage = $state(0);
  let originalSize = $state(false);
  let theme = $state<ThemeMode>("auto");
  let filter = $state<Filter>("all");
  let pathFilter = $state("");
  let showSummary = $state(false);

  let pending = $state<Record<string, FormState>>({});
  let saved = $state.raw<Record<string, FormState>>({});
  let pageInput = $state("1");

  let filteredImages = $derived.by(() => {
    let result = images;
    if (filter === "flagged") result = result.filter((p) => saved[p]?.severity);
    else if (filter === "unflagged")
      result = result.filter((p) => !saved[p]?.severity);
    const needle = pathFilter.trim().toLowerCase();
    if (needle) {
      result = result.filter((p) => p.toLowerCase().includes(needle));
    }
    return result;
  });

  let totalPages = $derived(
    Math.max(1, Math.ceil(filteredImages.length / gridSize)),
  );
  let visiblePaths = $derived(
    filteredImages.slice(currentPage * gridSize, (currentPage + 1) * gridSize),
  );
  let gridColumns = $derived({ 1: 1, 2: 2, 4: 2, 8: 4 }[gridSize]);
  let pendingCount = $derived(
    visiblePaths.filter((p) => pending[p]?.severity).length,
  );
  let canSave = $derived(pendingCount > 0);
  let summaryEntries = $derived(
    images
      .filter((p) => saved[p]?.severity)
      .map((p) => ({ path: p, ann: saved[p] })),
  );

  // Seed pending only on first sighting of each path so unsaved edits survive pagination.
  $effect(() => {
    for (const path of visiblePaths) {
      if (pending[path] === undefined) {
        pending[path] = saved[path]
          ? { ...saved[path] }
          : { severity: "", reason: "" };
      }
    }
  });

  $effect(() => {
    const effective =
      theme === "auto"
        ? window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light"
        : theme;
    document.documentElement.dataset.theme = effective;
  });

  $effect(() => {
    pageInput = String(currentPage + 1);
  });

  // Clamp when filter/grid changes shrink totalPages below currentPage.
  $effect(() => {
    if (currentPage > totalPages - 1) {
      currentPage = Math.max(0, totalPages - 1);
    }
  });

  function buildProjectPath(destFolder: string): string {
    return `${destFolder}/${sanitizeSegment(project)}_${sanitizeSegment(reviewer)}.radqc.yaml`;
  }

  async function loadExistingProject(path: string) {
    try {
      const existing = await invoke<{
        radqc: string;
        annotations: Record<string, { severity: string; reason: string }>;
      } | null>("read_project", { path });
      if (existing && existing.annotations) {
        const loaded: Record<string, FormState> = {};
        for (const [p, a] of Object.entries(existing.annotations)) {
          loaded[p] = {
            severity: a.severity as Severity,
            reason: a.reason,
          };
        }
        saved = loaded;
        const n = Object.keys(loaded).length;
        if (n > 0) {
          notice = `Loaded ${n} existing annotation${n === 1 ? "" : "s"} from ${path}.`;
        }
      } else {
        saved = {};
      }
    } catch (e) {
      notice = `Note: couldn't read existing project file (${String(e)}). Starting fresh.`;
      saved = {};
    }
  }

  async function openProject() {
    error = null;
    notice = null;
    const selected = await open({
      multiple: false,
      filters: [{ name: "RadQC project", extensions: ["yaml", "yml"] }],
    });
    if (typeof selected !== "string") return;

    try {
      const existing = await invoke<{
        radqc: string;
        reviewer: string;
        project: string;
        image_dir: string;
        annotations: Record<string, { severity: string; reason: string }>;
      } | null>("read_project", { path: selected });

      if (!existing) {
        error = `File not found: ${selected}`;
        return;
      }

      let imgs: string[];
      try {
        imgs = await invoke<string[]>("list_images", {
          folder: existing.image_dir,
        });
      } catch (e) {
        error = `Project loaded but its image folder is unreachable (${existing.image_dir}): ${String(e)}`;
        return;
      }

      reviewer = existing.reviewer;
      project = existing.project;
      folder = existing.image_dir;
      projectPath = selected;
      images = imgs;

      const loaded: Record<string, FormState> = {};
      for (const [p, a] of Object.entries(existing.annotations || {})) {
        loaded[p] = {
          severity: a.severity as Severity,
          reason: a.reason,
        };
      }
      saved = loaded;
      pending = {};
      currentPage = 0;
      filter = "all";
      pathFilter = "";
      started = true;

      const n = Object.keys(loaded).length;
      notice = `Loaded project: ${n} existing annotation${n === 1 ? "" : "s"}.`;
    } catch (e) {
      error = `Couldn't open project: ${String(e)}`;
    }
  }

  async function pickFolder() {
    error = null;
    notice = null;
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    folder = selected;
    projectPath = buildProjectPath(selected);
    currentPage = 0;
    pending = {};
    saved = {};
    pathFilter = "";
    try {
      images = await invoke<string[]>("list_images", { folder: selected });
    } catch (e) {
      error = String(e);
      images = [];
      return;
    }
    await loadExistingProject(projectPath);
  }

  async function changeOutputFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    projectPath = buildProjectPath(selected);
    pending = {};
    await loadExistingProject(projectPath);
  }

  function setGridSize(n: GridSize) {
    const firstVisible = currentPage * gridSize;
    gridSize = n;
    currentPage = Math.floor(firstVisible / n);
  }

  function goPrev() {
    if (currentPage > 0) currentPage -= 1;
  }
  function goNext() {
    if (currentPage < totalPages - 1) currentPage += 1;
  }

  function jumpToPage() {
    const n = parseInt(pageInput, 10);
    if (isNaN(n)) {
      pageInput = String(currentPage + 1);
      return;
    }
    const clamped = Math.max(1, Math.min(totalPages, n));
    currentPage = clamped - 1;
    pageInput = String(clamped);
  }

  function jumpToPath(path: string) {
    const isFlagged = !!saved[path]?.severity;
    if (
      (filter === "unflagged" && isFlagged) ||
      (filter === "flagged" && !isFlagged)
    ) {
      filter = "all";
    }
    // After potential filter change, locate the path in the (possibly updated) source.
    const sourceList = filter === "all" ? images : filteredImages;
    const idx = sourceList.indexOf(path);
    if (idx === -1) return;
    currentPage = Math.floor(idx / gridSize);
    showSummary = false;
  }

  async function savePage() {
    if (!folder || !projectPath) return;
    error = null;
    notice = null;
    const toSave = visiblePaths.filter((p) => pending[p]?.severity);

    for (const p of toSave) {
      if (!pending[p].reason.trim()) {
        error = `Reason required for "${p}".`;
        return;
      }
    }

    const newSaved: Record<string, FormState> = { ...saved };
    for (const p of toSave) {
      newSaved[p] = { ...pending[p] };
    }

    try {
      const annotationsForRust: Record<string, { severity: string; reason: string }> = {};
      for (const [p, a] of Object.entries(newSaved)) {
        annotationsForRust[p] = { severity: a.severity, reason: a.reason };
      }
      await invoke("save_project", {
        path: projectPath,
        reviewer,
        project,
        imageDir: folder,
        annotations: annotationsForRust,
      });
      saved = newSaved;
    } catch (e) {
      error = String(e);
      return;
    }

    if (currentPage < totalPages - 1) currentPage += 1;
  }

  async function backToSetup() {
    const hasUnsaved = visiblePaths.some((p) => {
      const pen = pending[p];
      const sav = saved[p];
      if (!pen?.severity) return false;
      if (!sav) return true;
      return pen.severity !== sav.severity || pen.reason !== sav.reason;
    });
    if (hasUnsaved) {
      const ok = await confirm(
        "You have unsaved annotations on this page. Going back to setup will discard them. Continue?",
        { title: "Discard unsaved annotations?", kind: "warning" },
      );
      if (!ok) return;
    }
    started = false;
    showSummary = false;
    pending = {};
    currentPage = 0;
    filter = "all";
  }

  async function openLink(url: string) {
    try {
      await openUrl(url);
    } catch (e) {
      error = `Couldn't open ${url}: ${String(e)}`;
    }
  }

  function imageUrl(path: string): string {
    return folder ? convertFileSrc(`${folder}/${path}`) : "";
  }
</script>

{#snippet definitions()}
  <p>
    <strong>Minor</strong> — image has a quality issue but is still usable for
    diagnosis or research. Worth noting but not excluding.
  </p>
  <p>
    <strong>Major</strong> — image has a quality issue that makes it unsuitable
    for diagnosis or research and should be excluded.
  </p>
{/snippet}

<main class="container">
  <header class="topbar">
    <div class="topbar-left">
      {#if started}
        <button
          class="back-btn"
          onclick={backToSetup}
          aria-label="Back to setup"
        >
          ← Setup
        </button>
      {/if}
      <h1 class="title">
        RadQC<span class="version">v{VERSION}</span><span class="beta"
          >Beta</span
        >
      </h1>
    </div>
    <div class="topbar-right">
      <div class="theme-toggle" role="group" aria-label="Theme">
        {#each ["auto", "light", "dark"] as t (t)}
          <button
            class={["seg", { active: theme === t }]}
            onclick={() => (theme = t as ThemeMode)}
          >
            {t[0].toUpperCase() + t.slice(1)}
          </button>
        {/each}
      </div>
      <details class="app-help">
        <summary class="help-btn" aria-label="About and links">?</summary>
        <div class="help-content app-help-content">
          <h3>About</h3>
          <p>
            RadQC v{VERSION} (Beta) — manual quality-control annotation tool
            for radiology images.
          </p>
          <p class="muted">© 2026 Neil Scheidwasser · Apache-2.0</p>
          <h3>Links</h3>
          <button class="link-btn" onclick={() => openLink(REPO_URL)}>
            ↗ GitHub repository
          </button>
          <button class="link-btn" onclick={() => openLink(ISSUES_URL)}>
            ↗ Report an issue
          </button>
        </div>
      </details>
    </div>
  </header>

  {#if !started}
    {#if mode === "choose"}
      <section class="landing">
        <div class="landing-cards">
          <article class="landing-card">
            <h2>Create a new project</h2>
            <p>Initialize a new annotation session.</p>
            <button class="primary" onclick={() => (mode = "new")}>
              Get started
            </button>
          </article>
          <article class="landing-card">
            <h2>Open existing project</h2>
            <p>
              Resume an annotation session from a previously saved
              <code>.radqc.yaml</code> project file.
            </p>
            <button class="primary" onclick={openProject}>
              Open project…
            </button>
          </article>
        </div>

        <details class="definitions">
          <summary>What is RadQC?</summary>
          <div class="definitions-body">
            <p>
              RadQC helps you triage radiology images for quality issues. For
              each image, decide whether it has a <strong>Minor</strong> or
              <strong>Major</strong> issue (or skip if it's fine), describe
              the issue briefly, and continue.
            </p>
            <p>
              Your annotations are saved to a YAML file on your machine —
              nothing is uploaded anywhere.
            </p>
          </div>
        </details>
        <details class="definitions">
          <summary>What do Minor and Major mean?</summary>
          <div class="definitions-body">
            {@render definitions()}
          </div>
        </details>
      </section>
    {:else}
      <section class="setup">
        <button class="link back-link" onclick={() => (mode = "choose")}>
          ← Back
        </button>

        <div class="row">
          <label>
            Reviewer ID
            <input bind:value={reviewer} placeholder="your initials or id" />
          </label>
          <label>
            Project
            <input bind:value={project} />
          </label>
        </div>

        <div class="picker">
          <span class="picker-label">Image folder</span>
          {#if folder}
            <div class="picker-row">
              <code class="picker-path" title={folder}>{folder}</code>
              <button class="link" onclick={pickFolder}>change…</button>
            </div>
            <span class="picker-info">
              {#if images.length === 0}
                <span class="warn">
                  No images found — pick a different folder.
                </span>
              {:else}
                {images.length} image{images.length === 1 ? "" : "s"} found
              {/if}
            </span>
          {:else}
            <button onclick={pickFolder} disabled={!reviewer || !project}>
              Select folder…
            </button>
          {/if}
        </div>

        <div class="picker">
          <span class="picker-label">Output folder</span>
          {#if projectPath}
            <div class="picker-row">
              <code class="picker-path" title={projectPath}>{projectPath}</code>
              <button class="link" onclick={changeOutputFolder}>change…</button>
            </div>
          {:else}
            <span class="picker-info muted">
              Defaults to the image folder. Pick image folder first to override.
            </span>
          {/if}
        </div>

        <button
          class="primary"
          onclick={() => (started = true)}
          disabled={!reviewer || !project || !folder || images.length === 0}
        >
          Start annotating
        </button>
      </section>
    {/if}
  {:else}
    <div class="meta-bar">
      <span class="meta">{reviewer} / {project}</span>
      <span class="counter">
        Page
        <input
          type="text"
          inputmode="numeric"
          bind:value={pageInput}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              jumpToPage();
              (e.currentTarget as HTMLInputElement).blur();
            }
          }}
          onblur={jumpToPage}
          class="page-input"
          aria-label="Jump to page"
        />
        / {totalPages}
        <span class="muted">· {filteredImages.length} images</span>
      </span>
      <input
        class="path-filter"
        type="text"
        placeholder="filter by path…"
        bind:value={pathFilter}
        aria-label="Filter images by path substring"
      />
      <details class="help">
        <summary class="help-btn" aria-label="What do Minor and Major mean?"
          >?</summary
        >
        <div class="help-content">
          {@render definitions()}
        </div>
      </details>
      <button
        class="summary-btn"
        onclick={() => (showSummary = !showSummary)}
        aria-label="Saved annotations"
      >
        <span>Saved</span>
        <span class="summary-count">{summaryEntries.length}</span>
      </button>
      <div class="controls">
        <span class="ctrl-label">Show</span>
        <div class="seg-group">
          {#each [["all", "All"], ["flagged", "Flagged"], ["unflagged", "Unflagged"]] as [f, label] (f)}
            <button
              class={["seg", { active: filter === f }]}
              onclick={() => (filter = f as Filter)}
            >
              {label}
            </button>
          {/each}
        </div>
        <span class="ctrl-label">Grid</span>
        <div class="seg-group">
          {#each [1, 2, 4, 8] as n (n)}
            <button
              class={["seg", { active: gridSize === n }]}
              onclick={() => setGridSize(n as GridSize)}
            >
              {n}
            </button>
          {/each}
        </div>
        {#if gridSize === 1}
          <label class="og-toggle">
            <input type="checkbox" bind:checked={originalSize} />
            Show in original size
          </label>
        {/if}
      </div>
    </div>

    <div class="output-bar">
      <span class="muted">Saving to</span>
      <code class="output-path" title={projectPath ?? ""}>{projectPath}</code>
    </div>

    {#if filteredImages.length === 0}
      <p class="info-box">
        {#if filter === "flagged"}
          No flagged images yet. Switch back to "All" to start annotating.
        {:else}
          All images have been annotated. Switch to "All" or "Flagged" to
          review.
        {/if}
      </p>
    {/if}

    <div
      class="grid"
      style:grid-template-columns="repeat({gridColumns}, minmax(0, 1fr))"
    >
      {#each visiblePaths as path (path)}
        <article class="cell">
          <div
            class={["cell-image", { scroll: gridSize === 1 && originalSize }]}
          >
            <img
              src={imageUrl(path)}
              alt={path}
              class={{ og: gridSize === 1 && originalSize }}
            />
          </div>
          <p class="cell-path"><code>{path}</code></p>
          {#if pending[path]}
            <fieldset class="flag">
              <legend>Flag this image</legend>
              <label>
                <input
                  type="radio"
                  name={`sev-${path}`}
                  value="minor"
                  bind:group={pending[path].severity}
                />
                Minor
              </label>
              <label>
                <input
                  type="radio"
                  name={`sev-${path}`}
                  value="major"
                  bind:group={pending[path].severity}
                />
                Major
              </label>
              {#if pending[path].severity}
                <button
                  type="button"
                  class="link"
                  onclick={() => {
                    pending[path].severity = "";
                    pending[path].reason = "";
                  }}
                >
                  clear
                </button>
              {/if}
            </fieldset>
            {#if pending[path].severity}
              <label class="reason-field">
                Reason
                <textarea
                  bind:value={pending[path].reason}
                  rows="2"
                  placeholder="describe the quality issue"
                  class={{ invalid: !pending[path].reason.trim() }}
                ></textarea>
              </label>
            {/if}
          {/if}
          {#if saved[path]}
            <p class="saved-tag">
              ✓ Saved as <strong>{saved[path].severity}</strong>
            </p>
          {/if}
        </article>
      {/each}
    </div>

    <div class="actions">
      <button onclick={goPrev} disabled={currentPage === 0}>← Prev page</button>
      <button class="primary" onclick={savePage} disabled={!canSave}>
        {canSave
          ? `Save ${pendingCount} annotation${pendingCount === 1 ? "" : "s"}`
          : "Save annotations"}
      </button>
      <button onclick={goNext} disabled={currentPage === totalPages - 1}>
        Skip →
      </button>
    </div>

    {#if showSummary}
      <aside class="summary-panel">
        <header class="summary-header">
          <h2>Saved annotations ({summaryEntries.length})</h2>
          <button class="link" onclick={() => (showSummary = false)}>
            close
          </button>
        </header>
        {#if summaryEntries.length === 0}
          <p class="muted">No annotations saved yet.</p>
        {:else}
          <table class="summary-table">
            <thead>
              <tr>
                <th>Flag</th>
                <th>Path</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              {#each summaryEntries as { path, ann } (path)}
                <tr class="summary-row" onclick={() => jumpToPath(path)}>
                  <td>
                    <span class="summary-severity sev-{ann.severity}">
                      {ann.severity}
                    </span>
                  </td>
                  <td><code class="summary-path">{path}</code></td>
                  <td class="summary-reason">{ann.reason}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </aside>
    {/if}
  {/if}

  {#if notice}
    <p class="notice">{notice}</p>
  {/if}
  {#if error}
    <p class="error">{error}</p>
  {/if}

  <footer class="footer">
    <p class="disclaimer">
      RadQC is a research tool for recording manual quality-control
      annotations on radiology images. It performs no automated analysis,
      makes no clinical interpretations, and is not a medical device or
      diagnostic tool. Users are responsible for any decisions made using
      its outputs. RadQC is shared for research purposes only and is not
      meant to be used for clinical practice.
    </p>
    <p class="copyright">
      © 2026 Neil Scheidwasser. Licensed under Apache-2.0.
    </p>
  </footer>
</main>

<style>
  :root {
    --bg: #fafaf9;
    --fg: #1c1917;
    --surface: #ffffff;
    --surface-2: #f5f5f4;
    --border: #e7e5e4;
    --muted: #78716c;
    --primary: #059669;
    --primary-fg: #ffffff;
    --primary-hover: #047857;
    --success: #16a34a;
    --warning: #d97706;
    --error: #dc2626;
    --invalid-border: #d97706;
    --disabled-bg: #f5f5f4;
    --disabled-fg: #a8a29e;
    --shadow: 0 1px 2px rgba(28, 25, 23, 0.05), 0 2px 6px rgba(28, 25, 23, 0.04);
    --radius: 8px;
    --radius-sm: 6px;
    font-family:
      ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, Inter,
      sans-serif;
    font-size: 15px;
    line-height: 1.5;
    color: var(--fg);
    background: var(--bg);
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
      --bg: #1c1917;
      --fg: #fafaf9;
      --surface: #292524;
      --surface-2: #44403c;
      --border: #44403c;
      --muted: #a8a29e;
      --primary: #10b981;
      --primary-fg: #ffffff;
      --primary-hover: #34d399;
      --success: #4ade80;
      --warning: #fbbf24;
      --error: #ef4444;
      --invalid-border: #f59e0b;
      --disabled-bg: #292524;
      --disabled-fg: #57534e;
      --shadow: 0 1px 2px rgba(0, 0, 0, 0.3), 0 4px 12px rgba(0, 0, 0, 0.18);
    }
  }

  :root[data-theme="dark"] {
    --bg: #1c1917;
    --fg: #fafaf9;
    --surface: #292524;
    --surface-2: #44403c;
    --border: #44403c;
    --muted: #a8a29e;
    --primary: #10b981;
    --primary-fg: #ffffff;
    --primary-hover: #34d399;
    --success: #4ade80;
    --warning: #fbbf24;
    --error: #ef4444;
    --invalid-border: #f59e0b;
    --disabled-bg: #292524;
    --disabled-fg: #57534e;
    --shadow: 0 1px 2px rgba(0, 0, 0, 0.3), 0 4px 12px rgba(0, 0, 0, 0.18);
  }

  :root[data-theme="light"] {
    --bg: #fafaf9;
    --fg: #1c1917;
    --surface: #ffffff;
    --surface-2: #f5f5f4;
    --border: #e7e5e4;
    --muted: #78716c;
    --primary: #059669;
    --primary-fg: #ffffff;
    --primary-hover: #047857;
    --success: #16a34a;
    --warning: #d97706;
    --error: #dc2626;
    --invalid-border: #d97706;
    --disabled-bg: #f5f5f4;
    --disabled-fg: #a8a29e;
    --shadow: 0 1px 2px rgba(28, 25, 23, 0.05), 0 2px 6px rgba(28, 25, 23, 0.04);
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    background: var(--bg);
    color: var(--fg);
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem 1.25rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }

  .topbar-left,
  .topbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .title {
    margin: 0;
    font-size: 1.4rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .version {
    font-size: 0.7rem;
    font-weight: 400;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .beta {
    font-size: 0.62rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 0.12rem 0.45rem;
    background: color-mix(in srgb, var(--primary) 18%, transparent);
    color: var(--primary);
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--primary) 30%, transparent);
  }

  .back-btn {
    background: transparent;
    border: 1px solid var(--border);
    padding: 0.3rem 0.7rem;
    font-size: 0.85rem;
    color: var(--muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .back-btn:hover {
    color: var(--fg);
    background: var(--surface-2);
  }

  .theme-toggle,
  .seg-group {
    display: inline-flex;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    padding: 0.15rem;
    gap: 0.1rem;
  }

  .seg {
    background: transparent;
    border: 1px solid transparent;
    padding: 0.25rem 0.7rem;
    font-size: 0.85rem;
    font-family: inherit;
    color: var(--muted);
    cursor: pointer;
    border-radius: calc(var(--radius-sm) - 0.15rem);
    transition: background-color 0.12s, color 0.12s;
  }

  .seg:hover:not(.active) {
    color: var(--fg);
  }

  .seg.active {
    background: var(--surface);
    color: var(--fg);
    box-shadow: var(--shadow);
  }

  .setup,
  .empty {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-width: 520px;
    margin: 4rem auto 0;
    padding: 1.5rem;
    background: var(--surface);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
  }

  .landing {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    margin: 1.5rem 0 0;
  }

  .landing-cards {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  .landing-card {
    background: var(--surface);
    border-radius: var(--radius);
    padding: 1.5rem;
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .landing-card h2 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }

  .landing-card p {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
    flex: 1;
  }

  .landing-card button {
    align-self: flex-start;
  }

  .back-link {
    align-self: flex-start;
    margin: 0 0 0.25rem;
    text-decoration: none !important;
  }

  @media (max-width: 600px) {
    .landing-cards {
      grid-template-columns: 1fr;
    }
  }

  .row {
    display: flex;
    gap: 0.75rem;
  }

  .row label,
  .reason-field {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  input[type="text"],
  input:not([type]),
  textarea,
  button {
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    padding: 0.55rem 0.75rem;
    font-size: 0.95rem;
    font-family: inherit;
    color: var(--fg);
    background: var(--surface);
    transition:
      border-color 0.15s,
      background-color 0.15s,
      color 0.15s;
  }

  input[type="text"]:focus,
  input:not([type]):focus,
  textarea:focus {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 20%, transparent);
  }

  textarea {
    resize: vertical;
    min-height: 2.5rem;
  }

  textarea.invalid {
    border-color: var(--invalid-border);
  }

  button {
    cursor: pointer;
    background: var(--surface-2);
  }

  button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--surface-2) 60%, var(--border));
  }

  button:disabled {
    background: var(--disabled-bg);
    color: var(--disabled-fg);
    cursor: not-allowed;
  }

  button.primary {
    background: var(--primary);
    color: var(--primary-fg);
    border-color: transparent;
    font-weight: 500;
  }

  button.primary:hover:not(:disabled) {
    background: var(--primary-hover);
  }

  button.primary:disabled {
    background: color-mix(in srgb, var(--primary) 25%, var(--disabled-bg));
    color: var(--primary-fg);
  }

  .meta-bar,
  .output-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background: var(--surface);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    font-size: 0.875rem;
    flex-wrap: wrap;
  }

  .output-bar {
    padding: 0.4rem 0.75rem;
    font-size: 0.8rem;
  }

  .output-path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--fg);
    font-size: 0.8rem;
  }

  .meta {
    font-weight: 500;
  }

  .counter {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
  }

  input.page-input {
    width: 3.2em;
    text-align: center;
    padding: 0.15rem 0.35rem;
    font-size: 0.85rem;
    margin: 0;
    font-variant-numeric: tabular-nums;
  }

  input.path-filter {
    width: 14rem;
    padding: 0.25rem 0.55rem;
    font-size: 0.85rem;
    margin: 0;
  }

  .muted {
    color: var(--muted);
  }

  .controls {
    margin-left: auto;
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .ctrl-label {
    color: var(--muted);
    font-size: 0.8rem;
  }

  .og-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--muted);
    cursor: pointer;
  }

  .og-toggle input {
    margin: 0;
  }

  .picker {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .picker-label {
    font-weight: 500;
    color: var(--fg);
  }

  .picker-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .picker-path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--fg);
    font-size: 0.8rem;
  }

  .picker-info {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .picker-info .warn {
    color: var(--invalid-border);
  }

  .picker > button {
    align-self: flex-start;
  }

  details.definitions {
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    padding: 0;
    border: 1px solid var(--border);
    font-size: 0.85rem;
  }

  details.definitions summary {
    cursor: pointer;
    padding: 0.5rem 0.75rem;
    color: var(--fg);
    user-select: none;
  }

  details.definitions[open] summary {
    border-bottom: 1px solid var(--border);
  }

  .definitions-body {
    padding: 0.5rem 0.75rem;
    color: var(--muted);
  }

  .definitions-body p {
    margin: 0.4rem 0;
  }

  .definitions-body p strong {
    color: var(--fg);
  }

  details.help,
  details.app-help {
    position: relative;
  }

  details.help summary.help-btn,
  details.app-help summary.help-btn {
    list-style: none;
    cursor: pointer;
    width: 1.6rem;
    height: 1.6rem;
    border-radius: 50%;
    background: var(--surface-2);
    color: var(--muted);
    font-size: 0.85rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    user-select: none;
    transition: background-color 0.12s, color 0.12s;
    border: 1px solid var(--border);
  }

  details.help summary.help-btn::-webkit-details-marker,
  details.app-help summary.help-btn::-webkit-details-marker {
    display: none;
  }

  details.help summary.help-btn:hover,
  details.help[open] summary.help-btn,
  details.app-help summary.help-btn:hover,
  details.app-help[open] summary.help-btn {
    background: var(--primary);
    color: var(--primary-fg);
    border-color: transparent;
  }

  details.help .help-content,
  details.app-help .help-content {
    position: absolute;
    top: calc(100% + 0.4rem);
    right: 0;
    width: 22rem;
    max-width: 90vw;
    padding: 0.85rem 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    font-size: 0.85rem;
    color: var(--muted);
    z-index: 50;
  }

  .help-content h3 {
    margin: 0.6rem 0 0.3rem;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    font-weight: 600;
  }

  .help-content h3:first-child {
    margin-top: 0;
  }

  .help-content p {
    margin: 0.3rem 0;
    color: var(--fg);
  }

  .help-content p.muted {
    color: var(--muted);
    font-size: 0.8rem;
  }

  .help-content p strong {
    color: var(--fg);
  }

  .link-btn {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid var(--border);
    padding: 0.4rem 0.65rem;
    margin: 0.25rem 0;
    font-size: 0.85rem;
    color: var(--fg);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .link-btn:hover {
    background: var(--surface-2);
  }

  .summary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.6rem;
    font-size: 0.85rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--fg);
    cursor: pointer;
  }

  .summary-btn:hover {
    background: color-mix(in srgb, var(--surface-2) 60%, var(--border));
  }

  .summary-count {
    font-size: 0.75rem;
    background: var(--primary);
    color: var(--primary-fg);
    padding: 0.05rem 0.45rem;
    border-radius: 999px;
    font-variant-numeric: tabular-nums;
    font-weight: 500;
  }

  .summary-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 28rem;
    max-width: 90vw;
    background: var(--surface);
    border-left: 1px solid var(--border);
    box-shadow: -4px 0 14px rgba(0, 0, 0, 0.12);
    overflow-y: auto;
    padding: 1rem 1.25rem;
    z-index: 200;
  }

  .summary-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
    margin-bottom: 0.5rem;
    position: sticky;
    top: 0;
    background: var(--surface);
  }

  .summary-header h2 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .summary-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
    table-layout: fixed;
  }

  .summary-table th {
    text-align: left;
    font-weight: 500;
    color: var(--muted);
    padding: 0.4rem 0.4rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.7rem;
    border-bottom: 1px solid var(--border);
  }

  .summary-table th:nth-child(1) {
    width: 4.5rem;
  }

  .summary-table th:nth-child(2) {
    width: 38%;
  }

  .summary-table td {
    padding: 0.45rem 0.4rem;
    border-bottom: 1px solid var(--border);
    vertical-align: top;
    word-break: break-word;
  }

  .summary-table tr.summary-row {
    cursor: pointer;
  }

  .summary-table tr.summary-row:hover td {
    background: var(--surface-2);
  }

  .summary-severity {
    display: inline-block;
    padding: 0.08rem 0.4rem;
    border-radius: var(--radius-sm);
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    flex-shrink: 0;
  }

  .summary-severity.sev-minor {
    background: color-mix(in srgb, var(--warning) 18%, transparent);
    color: var(--warning);
  }

  .summary-severity.sev-major {
    background: color-mix(in srgb, var(--error) 18%, transparent);
    color: var(--error);
  }

  .summary-path {
    font-size: 0.78rem;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .summary-reason {
    color: var(--fg);
    word-break: break-word;
  }

  .grid {
    display: grid;
    gap: 1rem;
  }

  .cell {
    background: var(--surface);
    border-radius: var(--radius);
    padding: 0.75rem;
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  .cell-image {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    padding: 0.5rem;
    overflow: hidden;
    min-height: 8rem;
  }

  .cell-image.scroll {
    overflow: auto;
    max-height: 65vh;
    align-items: flex-start;
    justify-content: flex-start;
  }

  .cell-image img {
    max-width: 100%;
    max-height: 55vh;
    object-fit: contain;
  }

  .cell-image img.og {
    max-width: none;
    max-height: none;
    object-fit: none;
  }

  .cell-path {
    margin: 0;
    text-align: center;
    color: var(--muted);
    word-break: break-all;
  }

  .cell-path code,
  .output-path {
    font-size: 0.8rem;
  }

  fieldset.flag {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.75rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
    margin: 0;
  }

  fieldset.flag legend {
    padding: 0 0.4rem;
    font-size: 0.78rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  fieldset.flag label {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.9rem;
    cursor: pointer;
    color: var(--fg);
  }

  fieldset.flag input[type="radio"] {
    margin: 0;
  }

  button.link {
    background: transparent;
    border: none;
    padding: 0;
    margin-left: auto;
    color: var(--muted);
    font-size: 0.8rem;
    text-decoration: underline;
    cursor: pointer;
  }

  button.link:hover {
    color: var(--fg);
    background: transparent;
  }

  .saved-tag {
    margin: 0;
    font-size: 0.85rem;
    color: var(--success);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    padding-top: 0.25rem;
  }

  .actions button {
    flex: 1;
  }

  .info-box {
    margin: 0;
    padding: 1rem 1.25rem;
    background: var(--surface);
    border: 1px dashed var(--border);
    border-radius: var(--radius);
    color: var(--muted);
    text-align: center;
    font-size: 0.9rem;
  }

  .notice {
    margin: 0;
    padding: 0.5rem 0.85rem;
    background: color-mix(in srgb, var(--primary) 10%, var(--surface));
    color: var(--fg);
    border: 1px solid color-mix(in srgb, var(--primary) 25%, transparent);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
  }

  .error {
    margin: 0;
    padding: 0.6rem 0.9rem;
    background: color-mix(in srgb, var(--error) 12%, var(--surface));
    color: var(--error);
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--error) 30%, transparent);
    font-size: 0.9rem;
  }

  h1 {
    margin: 0;
  }

  .footer {
    margin-top: 0.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
    font-size: 0.75rem;
    color: var(--muted);
    line-height: 1.5;
  }

  .footer p {
    margin: 0.4rem 0;
  }

  .footer .disclaimer {
    max-width: 70ch;
  }

  .footer .copyright {
    color: var(--muted);
  }
</style>
