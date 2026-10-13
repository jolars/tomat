import { readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

export async function buildSitemap(bookDirectory, baseUrl = "https://tomat.cc/") {
  const base = new URL(`${baseUrl.replace(/\/+$/, "")}/`);
  const urls = new Set();

  async function visit(directory = "") {
    const entries = await readdir(path.join(bookDirectory, directory), {
      withFileTypes: true,
    });
    for (const entry of entries) {
      const relativePath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        await visit(relativePath);
        continue;
      }
      if (!entry.isFile() || !entry.name.endsWith(".html")) continue;
      const relativeUrl = relativePath.split(path.sep).join("/");
      if (["404.html", "print.html", "toc.html"].includes(relativeUrl)) continue;
      const html = await readFile(path.join(bookDirectory, relativePath), "utf8");
      if (/http-equiv\s*=\s*["']refresh["']/i.test(html)) continue;
      const location = relativeUrl.replace(/(^|\/)index\.html$/, "$1");
      urls.add(new URL(location.split("/").map(encodeURIComponent).join("/"), base).href);
    }
  }

  await visit();
  const escapeXml = (value) => value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
  const entries = [...urls].sort().map((url) => `  <url><loc>${escapeXml(url)}</loc></url>`);
  await writeFile(
    path.join(bookDirectory, "sitemap.xml"),
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${entries.join("\n")}\n</urlset>\n`,
  );
}
