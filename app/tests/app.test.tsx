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
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.app.windows[0].label, "main");
  assert.equal(config.app.windows[0].title, "Mission Control");
  assert.ok(config.app.security.csp.includes("default-src 'self'"));
});
