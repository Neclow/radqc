export function sanitizeSegment(s: string): string {
  return s
    .replace(/[/\\:*?"<>|\x00]+/g, "_")
    .replace(/^\.+/, "")
    .replace(/[. ]+$/, "");
}
