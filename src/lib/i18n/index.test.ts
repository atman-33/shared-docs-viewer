import { describe, expect, it } from "vitest";
import { toLocale, translate } from "@/lib/i18n";
import { en, type MessageKey } from "@/lib/i18n/messages/en";
import { ja } from "@/lib/i18n/messages/ja";

describe("translate", () => {
  it("returns the Japanese text when the key is translated", () => {
    expect(translate("ja", "common.cancel")).toBe("キャンセル");
  });

  it("returns the English text for the en locale", () => {
    expect(translate("en", "common.cancel")).toBe("Cancel");
  });

  it("falls back to English for a key Japanese leaves out", () => {
    const key = (Object.keys(en) as MessageKey[]).find((k) => !(k in ja));
    if (key) expect(translate("ja", key)).toBe(en[key]);
  });
});

describe("translate interpolation", () => {
  it("fills a placeholder from vars", () => {
    expect(
      translate("en", "banner.update.available", { tag: "v1", version: "0" }),
    ).toBe("New version v1 is available (current v0)");
  });

  it("leaves a placeholder with no matching var untouched", () => {
    expect(translate("en", "banner.update.available", { tag: "v1" })).toBe(
      "New version v1 is available (current v{version})",
    );
  });

  it("interpolates the Japanese text the same way", () => {
    expect(
      translate("ja", "banner.update.available", { tag: "v1", version: "0" }),
    ).toBe("新しいバージョン v1 が利用できます（現在 v0）");
  });
});

describe("toLocale", () => {
  it("accepts the known locales", () => {
    expect(toLocale("ja")).toBe("ja");
    expect(toLocale("en")).toBe("en");
  });

  it("reads anything unknown as English", () => {
    expect(toLocale("fr")).toBe("en");
    expect(toLocale(undefined)).toBe("en");
    expect(toLocale("")).toBe("en");
  });
});
