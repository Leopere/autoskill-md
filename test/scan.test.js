import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { scanProject } from "../src/scan.js";

test("scanProject reads metadata, hints, and routes across supported languages", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "autoskill-scan-"));
  await mkdir(path.join(root, "src"), { recursive: true });

  await writeFile(
    path.join(root, "package.json"),
    JSON.stringify({
      name: "tickets-api",
      description: "This API lets agents read ticket status.",
      homepage: "https://example.com/docs"
    })
  );

  await writeFile(path.join(root, "go.mod"), "module github.com/example/tickets\n");
  await writeFile(
    path.join(root, "Cargo.toml"),
    '[package]\nname = "tickets-rs"\ndescription = "Ticket helpers."\n'
  );
  await writeFile(
    path.join(root, "pyproject.toml"),
    '[project]\nname = "tickets-py"\ndescription = "Ticket tools."\n'
  );

  await writeFile(
    path.join(root, "src", "routes.js"),
    `
// autoskill: auth: Public reads need no auth.
// autoskill: safe: GET ticket status by id.
app.get("/api/v1/tickets/:id", handler);
app.post("/api/v1/tickets", handler);
`
  );
  await writeFile(
    path.join(root, "src", "main.go"),
    'package main\nfunc main() { http.HandleFunc("/api/v1/health", h) }\n'
  );
  await writeFile(
    path.join(root, "src", "lib.rs"),
    '#[get("/api/v1/profile")]\nasync fn profile() {}\n'
  );
  await writeFile(
    path.join(root, "src", "app.py"),
    '@app.delete("/api/v1/tickets/{ticket_id}")\ndef delete_ticket(): pass\n'
  );

  const scan = await scanProject(root, {});

  assert.equal(scan.name, "tickets-api");
  assert.equal(scan.purpose, "This API lets agents read ticket status.");
  assert.equal(scan.apiBase, "/api/v1");
  assert.equal(scan.auth, "Public reads need no auth");
  assert.deepEqual(scan.languages, ["go", "node", "python", "rust"]);
  assert.ok(scan.docs.includes("https://example.com/docs"));
  assert.ok(scan.safeActions.includes("GET /api/v1/tickets/:id"));
  assert.ok(scan.safeActions.includes("GET ticket status by id"));
  assert.ok(scan.riskyActions.includes("POST /api/v1/tickets"));
  assert.ok(scan.riskyActions.includes("DELETE /api/v1/tickets/{ticket_id}"));
});
