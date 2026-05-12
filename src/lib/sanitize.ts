// Order matters: separators → leading dots → trailing dots/spaces. Reordering
// changes outputs (e.g. "../etc" relies on slash-replacement happening before
// leading-dot stripping).
export function sanitizeSegment(s: string): string {
  return s
    .replace(/[/\\:*?"<>|\x00]+/g, "_")
    .replace(/^\.+/, "")
    .replace(/[. ]+$/, "");
}
