import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import App from "../src/App";

test("renders the application identity in the main landmark", () => {
  const markup = renderToStaticMarkup(<App />);
  assert.match(markup, /<main[^>]*>/);
  assert.match(markup, /<h1>Mission Control<\/h1>/);
});

test("production uses bundled assets and a named native window", () => {
  const config = JSON.parse(
    readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  );
  assert.deepEqual(config.plugins.sql.preload, ["sqlite:mission-control.db"]);
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.app.windows[0].label, "main");
  assert.equal(config.app.windows[0].title, "Mission Control");
  assert.ok(config.app.security.csp.includes("default-src 'self'"));
});

test("isolates development identity and exposes no generic SQL or filesystem permissions", () => {
  const read = (path: string) => JSON.parse(readFileSync(new URL(path, import.meta.url), "utf8"));
  assert.equal(read("../src-tauri/tauri.conf.json").identifier, "com.missioncontrol.desktop");
  assert.equal(read("../src-tauri/tauri.dev.conf.json").identifier, "com.missioncontrol.desktop.dev");
  assert.deepEqual(read("../src-tauri/capabilities/default.json").permissions, []);
  assert.match(read("../package.json").scripts["desktop:dev"], /--config src-tauri\/tauri.dev.conf.json/);
});

test("desktop packaging selects native formats without changing identity or security", () => {
  const read = (name: string) => JSON.parse(readFileSync(new URL(`../src-tauri/${name}`, import.meta.url), "utf8"));
  const base = read("tauri.conf.json");
  assert.deepEqual(base.bundle.targets, ["app"]);
  for (const platform of ["windows", "linux"]) {
    const overlay = read(`tauri.${platform}.conf.json`);
    assert.deepEqual(Object.keys(overlay), ["bundle"]);
    assert.deepEqual(overlay.bundle.targets, [platform === "windows" ? "nsis" : "deb"]);
    if (platform === "windows") {
      assert.equal(overlay.bundle.windows.webviewInstallMode.type, "offlineInstaller");
    }
  }
  const scripts = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8")).scripts;
  assert.equal(scripts["desktop:build"], "tauri build");
});
