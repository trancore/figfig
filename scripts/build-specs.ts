import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { specs } from "../specs/index.ts";

const file = new URL("../assets/specs.json", import.meta.url);
const json = `${JSON.stringify(specs, null, 2)}\n`;
if (process.argv.includes("--check")) {
  if (readFileSync(file, "utf8") !== json) throw new Error("Run pnpm run specs to regenerate assets/specs.json");
} else {
  mkdirSync(new URL("../assets/", import.meta.url), { recursive: true });
  writeFileSync(file, json);
}
