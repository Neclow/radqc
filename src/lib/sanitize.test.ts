import { describe, expect, test } from "vitest";
import { sanitizeSegment } from "./sanitize";

describe("sanitizeSegment", () => {
  test("preserves safe input", () => {
    expect(sanitizeSegment("alice")).toBe("alice");
    expect(sanitizeSegment("default")).toBe("default");
    expect(sanitizeSegment("v1.0.0")).toBe("v1.0.0");
    expect(sanitizeSegment("study-2026_run3")).toBe("study-2026_run3");
  });

  test("replaces path separators", () => {
    expect(sanitizeSegment("a/b")).toBe("a_b");
    expect(sanitizeSegment("a\\b")).toBe("a_b");
    expect(sanitizeSegment("a/b\\c")).toBe("a_b_c");
  });

  test("replaces Windows-reserved characters", () => {
    expect(sanitizeSegment("a:b")).toBe("a_b");
    expect(sanitizeSegment("a*b?c")).toBe("a_b_c");
    expect(sanitizeSegment('a"b<c>d|e')).toBe("a_b_c_d_e");
  });

  test("replaces null bytes", () => {
    expect(sanitizeSegment("a\x00b")).toBe("a_b");
  });

  test("strips leading dots", () => {
    expect(sanitizeSegment(".hidden")).toBe("hidden");
    expect(sanitizeSegment("..foo")).toBe("foo");
    expect(sanitizeSegment("...bar")).toBe("bar");
  });

  test("strips trailing dots and spaces", () => {
    expect(sanitizeSegment("foo.")).toBe("foo");
    expect(sanitizeSegment("foo ")).toBe("foo");
    expect(sanitizeSegment("foo. .")).toBe("foo");
  });

  test("preserves interior dots", () => {
    expect(sanitizeSegment("v1.0")).toBe("v1.0");
    expect(sanitizeSegment("a.b.c")).toBe("a.b.c");
  });

  test("preserves non-ASCII letters", () => {
    expect(sanitizeSegment("Müller")).toBe("Müller");
    expect(sanitizeSegment("日本語")).toBe("日本語");
  });

  test("handles pathological inputs", () => {
    expect(sanitizeSegment("..")).toBe("");
    expect(sanitizeSegment("...")).toBe("");
    expect(sanitizeSegment("/")).toBe("_");
    expect(sanitizeSegment("../etc/passwd")).toBe("_etc_passwd");
  });

  test("handles empty string", () => {
    expect(sanitizeSegment("")).toBe("");
  });

  test("is idempotent", () => {
    const inputs = ["alice", "a/b", "..foo.", "/etc/passwd", "Müller", "..", ""];
    for (const input of inputs) {
      const once = sanitizeSegment(input);
      const twice = sanitizeSegment(once);
      expect(twice).toBe(once);
    }
  });
});
