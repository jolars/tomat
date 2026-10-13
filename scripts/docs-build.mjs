import { execFileSync } from "node:child_process";
import { chmod, copyFile, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { buildLegacyPages } from "./docs-legacy.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const book = new URL("../docs/book/", import.meta.url);
const installer = new URL("./tomat-installer.sh", import.meta.url);
const publishedInstaller = new URL("install", book);

execFileSync("mdbook", ["build", "docs"], { cwd: root, stdio: "inherit" });
await copyFile(installer, publishedInstaller);
await chmod(publishedInstaller, 0o644);
const [source, published] = await Promise.all([
  readFile(installer),
  readFile(publishedInstaller),
]);
if (!source.equals(published))
  throw new Error("Published installer differs from the source.");
execFileSync("sh", ["-n", fileURLToPath(publishedInstaller)], {
  cwd: root,
  stdio: "inherit",
});
await buildLegacyPages(
  fileURLToPath(book),
  fileURLToPath(new URL("../target/legacy-pages/", import.meta.url)),
);
