/**
 * Guards the bundled AI-edit skill (T-1001, T-1018).
 *
 * One `diagram-edit` skill edits every kind of diagram: it reads the note's
 * `type` and follows that kind's rule file, so the formats live in the rules
 * and not in the skill. The skill and its rules ship inside the binary
 * (`src-tauri/resources/`); this test pins that layout and the cross-links
 * between them.
 */

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { DIAGRAM_KINDS } from "@/lib/diagram-kinds";

const root = process.cwd();
const skillPath = join(root, "src-tauri", "resources", "skills", "diagram-edit", "SKILL.md");
const skill = readFileSync(skillPath, "utf8").replace(/\r\n/g, "\n");
const rulesDir = join(root, "src-tauri", "resources", "skill-rules");

/** The rule file each kind is documented in. */
const RULE_OF: Record<(typeof DIAGRAM_KINDS)[number], string> = {
  schedule: "schedules.md",
  mindmap: "mindmaps.md",
  matrix2x2: "diagrams.md",
  flow: "diagrams.md",
  pfd: "diagrams.md",
  algorithm: "diagrams.md",
  usecase: "diagrams.md",
  architecture: "diagrams.md",
  ifdam: "diagrams.md",
};

describe("diagram-edit skill", () => {
  it("is named diagram-edit and takes a path and an instruction", () => {
    expect(skill).toMatch(/^name: diagram-edit$/m);
    expect(skill).toMatch(/^argument-hint: "<diagram-file-path> <instruction>"$/m);
  });

  it.each(DIAGRAM_KINDS)("has a rule for %s, and the skill points at it", (kind) => {
    const rule = RULE_OF[kind];
    expect(existsSync(join(rulesDir, rule))).toBe(true);
    expect(skill).toContain(`\`${rule}\``);
    expect(skill).toContain(kind);
  });

  it("keeps formats out of the skill: no node, item or step grammar of its own", () => {
    expect(skill).not.toMatch(/^- \[bar\]/m);
    expect(skill).not.toMatch(/\bI-001\b/);
    expect(skill).not.toMatch(/^## (Items|Steps|Lanes|Edges|Non-working)\b/m);
  });

  it("states the shared rules every kind needs", () => {
    expect(skill).toContain("`node:`");
    expect(skill).toContain("## Memo");
    expect(skill).toMatch(/ids are never changed or reused/i);
    expect(skill).toMatch(/updated:/);
    expect(skill).toMatch(/Do not invent an id prefix/);
  });

  it("ships no per-kind skills beside it", () => {
    const dir = join(root, "src-tauri", "resources", "skills", "diagram-edit");
    for (const entry of readdirSync(dir)) {
      expect(entry).not.toMatch(/schedule-edit|mindmap-edit/);
    }
  });

  it("leaves no reference to the retired skills in the bundled rules", () => {
    for (const rule of new Set(Object.values(RULE_OF))) {
      const text = readFileSync(join(rulesDir, rule), "utf8");
      expect(text).not.toMatch(/schedule-edit|mindmap-edit/);
    }
  });
});
