import { copyFile, mkdir, readdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";

export function redirectTarget({ pathname, search = "", hash = "" }) {
  const target = new URL("https://tomat.cc/");
  target.pathname =
    pathname === "/tomat" ? "/" : pathname.replace(/^\/tomat(?=\/)/, "");
  target.search = search;
  target.hash = hash;
  return target.href;
}

function redirectPage(relativePath) {
  const fallback = new URL(
    ["index.html", "404.html"].includes(relativePath) ? "" : relativePath,
    "https://tomat.cc/",
  ).href;
  const escaped = fallback
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;");
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Tomat documentation has moved</title>
  <link rel="canonical" href="${escaped}">
  <script>window.location.replace((${redirectTarget.toString()})(window.location));</script>
  <meta http-equiv="refresh" content="0; url=${escaped}">
</head>
<body><p>Tomat documentation has moved to <a href="${escaped}">${escaped}</a>.</p></body>
</html>
`;
}

export async function buildLegacyPages(bookDirectory, outputDirectory) {
  await rm(outputDirectory, { recursive: true, force: true });
  await mkdir(outputDirectory, { recursive: true });

  async function visit(relativeDirectory = "") {
    const entries = await readdir(path.join(bookDirectory, relativeDirectory), {
      withFileTypes: true,
    });
    for (const entry of entries) {
      const relativePath = path.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) await visit(relativePath);
      else if (entry.isFile() && entry.name.endsWith(".html")) {
        const destination = path.join(outputDirectory, relativePath);
        await mkdir(path.dirname(destination), { recursive: true });
        await writeFile(
          destination,
          redirectPage(relativePath.split(path.sep).join("/")),
        );
      }
    }
  }

  await visit();
  // Shell clients do not follow browser redirects, so retain the installer.
  await copyFile(
    path.join(bookDirectory, "install"),
    path.join(outputDirectory, "install"),
  );
  await writeFile(path.join(outputDirectory, ".nojekyll"), "");
}
