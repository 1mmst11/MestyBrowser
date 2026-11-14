(function () {
  "use strict";

  const VARS = [
    "--bg", "--base", "--surface", "--surface-2", "--chip",
    "--text", "--ink", "--muted", "--ink-dim",
    "--card", "--card-soft", "--field", "--field-hover",
    "--border", "--shadow", "--highlight"
  ];

  function color(value, fallback) {
    return /^#[0-9a-f]{6}$/i.test(String(value || "")) ? value : fallback;
  }

  function luminance(hex) {
    const channels = [1, 3, 5].map((index) => {
      const value = parseInt(hex.slice(index, index + 2), 16) / 255;
      return value <= 0.03928 ? value / 12.92 : Math.pow((value + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  }

  function clear() {
    VARS.forEach((name) => document.documentElement.style.removeProperty(name));
  }

  function apply(custom) {
    const settings = custom || {};
    const base = color(settings.base, "#f6f7f9");
    const surface = color(settings.surface, "#ffffff");
    const ink = color(settings.ink, "#1d1f23");
    const muted = color(settings.muted, "#71757c");
    const dark = luminance(base) < 0.45;
    const style = document.documentElement.style;

    style.setProperty("--bg", base);
    style.setProperty("--base", base);
    style.setProperty("--surface", surface);
    style.setProperty("--card", surface);
    style.setProperty("--surface-2", "color-mix(in srgb, " + surface + " 88%, " + ink + ")");
    style.setProperty("--chip", "color-mix(in srgb, " + surface + " 82%, " + ink + ")");
    style.setProperty("--card-soft", "color-mix(in srgb, " + surface + " 86%, " + ink + ")");
    style.setProperty("--field", "color-mix(in srgb, " + surface + " 78%, " + base + ")");
    style.setProperty("--field-hover", "color-mix(in srgb, " + surface + " 68%, " + ink + ")");
    style.setProperty("--text", ink);
    style.setProperty("--ink", ink);
    style.setProperty("--muted", muted);
    style.setProperty("--ink-dim", muted);
    style.setProperty("--border", "color-mix(in srgb, " + ink + " 14%, transparent)");
    style.setProperty("--shadow", dark ? "rgba(0, 0, 0, 0.5)" : "rgba(24, 20, 40, 0.12)");
    style.setProperty("--highlight", dark ? "rgba(255, 255, 255, 0.05)" : "rgba(255, 255, 255, 0.65)");
  }

  window.mestyApplyTheme = function (theme, custom) {
    if (theme === "custom") apply(custom);
    else clear();
  };
})();
