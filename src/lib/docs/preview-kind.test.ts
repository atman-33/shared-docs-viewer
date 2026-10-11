import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { IMAGE_EXTENSIONS, isImageEntry, isPreviewable, isPreviewableOrImage, previewKindForPath, TEXT_EXTENSIONS } from "./preview-kind";
import type { DocsEntry } from "@/types";

function entry(over: Partial<DocsEntry>): DocsEntry {
  return {
    path: "G:/share/x",
    name: "x",
    is_dir: false,
    is_markdown: false,
    is_html: false,
    is_text: false,
    modified: 0,
    ...over,
  };
}

describe("previewKindForPath", () => {
  it("names the renderer for each kind it can show", () => {
    expect(previewKindForPath("G:/share/a.md")).toBe("markdown");
    expect(previewKindForPath("G:/share/a.MARKDOWN")).toBe("markdown");
    expect(previewKindForPath("G:/share/a.htm")).toBe("html");
    expect(previewKindForPath("G:/share/notes.txt")).toBe("text");
    expect(previewKindForPath("G:/share/data.YAML")).toBe("text");
  });

  it("names the image kinds the preview inlines", () => {
    expect(previewKindForPath("C:/vault/shot-01-base.png")).toBe("image");
    expect(previewKindForPath("C:/vault/shot-02-look0.JPG")).toBe("image");
    expect(previewKindForPath("C:/vault/diagram.Svg")).toBe("image");
    expect(previewKindForPath("C:/vault/photo.avif")).toBe("image");
  });

  it("returns null for what only the OS can open", () => {
    expect(previewKindForPath("G:/share/sheet.xlsx")).toBeNull();
    expect(previewKindForPath("G:/share/report.pdf")).toBeNull();
    expect(previewKindForPath("G:/share/README")).toBeNull();
  });

  it("reads the extension off the name, not the folders above it", () => {
    // A folder named `notes.md` holding `report.pdf` must not read as
    // Markdown — the last dot in the *path* is not the file's extension.
    expect(previewKindForPath("G:/share/notes.md/report.pdf")).toBeNull();
    expect(previewKindForPath(String.raw`G:\share\notes.md\report.pdf`)).toBeNull();
  });

  it("does not read a dotfile's name as an extension", () => {
    expect(previewKindForPath("G:/share/.gitignore")).toBeNull();
  });
});

describe("isPreviewable", () => {
  it("follows the backend's flags", () => {
    expect(isPreviewable(entry({ is_markdown: true }))).toBe(true);
    expect(isPreviewable(entry({ is_html: true }))).toBe(true);
    expect(isPreviewable(entry({ is_text: true }))).toBe(true);
    expect(isPreviewable(entry({}))).toBe(false);
  });

  it("is never true for a folder", () => {
    expect(isPreviewable(entry({ is_dir: true, is_text: true }))).toBe(false);
  });

  it("stays false for images — they have no backend flag", () => {
    expect(isPreviewable(entry({ name: "shot-01-base.png" }))).toBe(false);
  });
});

describe("isImageEntry", () => {
  it("names the image files the preview inlines", () => {
    expect(isImageEntry(entry({ name: "shot-01-base.png" }))).toBe(true);
    expect(isImageEntry(entry({ name: "shot-02-look0.JPG" }))).toBe(true);
  });

  it("is never true for documents, binaries or folders", () => {
    expect(isImageEntry(entry({ name: "a.md", is_markdown: true }))).toBe(false);
    expect(isImageEntry(entry({ name: "sheet.xlsx" }))).toBe(false);
    expect(isImageEntry(entry({ name: "shot.png", is_dir: true }))).toBe(false);
  });
});

describe("isPreviewableOrImage", () => {
  it("offers documents and images but no other binaries", () => {
    expect(isPreviewableOrImage(entry({ name: "a.md", is_markdown: true }))).toBe(true);
    expect(isPreviewableOrImage(entry({ name: "shot-01-base.png" }))).toBe(true);
    expect(isPreviewableOrImage(entry({ name: "sheet.xlsx" }))).toBe(false);
    expect(isPreviewableOrImage(entry({ name: "shot.png", is_dir: true }))).toBe(false);
  });
});

describe("the two copies of the text-extension list", () => {
  it("says the same thing on both sides of the IPC boundary", () => {
    // The tree's flags come from Rust and the preview judges a bare path, so
    // a kind added to one list and not the other makes a file clickable and
    // then unreadable. Cheaper to fail here than to find it in the app.
    const rust = readFileSync("src-tauri/src/docs.rs", "utf8");
    const block = /const TEXT_EXTENSIONS: &\[&str\] = &\[([\s\S]*?)\];/.exec(rust);
    expect(block, "TEXT_EXTENSIONS not found in docs.rs").not.toBeNull();
    const fromRust = [...(block?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]);
    expect(fromRust.length).toBeGreaterThan(0);
    expect([...fromRust].sort()).toEqual([...TEXT_EXTENSIONS].sort());
  });
});

describe("the two copies of the image-extension list", () => {
  it("says the same thing on both sides of the IPC boundary", () => {
    // The preview judges a bare path while `docs_read_asset` decides what it
    // inlines in `media_type`, so a kind added to one list and not the other
    // makes a file listed and then unreadable. Only the match arms name an
    // extension: the arms' right-hand sides are MIME types and the function
    // around them carries error prose.
    const rust = readFileSync("src-tauri/src/docs.rs", "utf8");
    const start = rust.indexOf("fn media_type");
    expect(start, "media_type not found in docs.rs").toBeGreaterThan(-1);
    const block = rust.slice(start, rust.indexOf("pub fn read_asset"));
    const fromRust = [...block.matchAll(/"([a-z0-9]+)"(?=\s*(?:\||=>))/g)].map((m) => m[1]);
    expect(fromRust.length).toBeGreaterThan(0);
    expect([...fromRust].sort()).toEqual([...IMAGE_EXTENSIONS].sort());
  });
});
