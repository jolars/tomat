import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const source = new URL("./tomat-installer.sh", import.meta.url);
const published = new URL("../docs/book/install", import.meta.url);
const [expected, actual] = await Promise.all([readFile(source), readFile(published)]);
if (!expected.equals(actual)) throw new Error("Published installer differs from the source.");
execFileSync("sh", ["-n", fileURLToPath(published)], { stdio: "inherit" });
