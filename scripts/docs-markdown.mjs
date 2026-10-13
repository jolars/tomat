import { execFileSync } from "node:child_process";
import { readdir, readFile, mkdir, writeFile, rm } from "node:fs/promises";
import path from "node:path";

export async function buildMarkdown(book) {
  await rm(path.join(book, "_markdown"), { recursive: true, force: true });
  async function visit(directory = "") {
    for (const entry of await readdir(path.join(book, directory), { withFileTypes: true })) {
      const relative = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        if (entry.name !== "_markdown") await visit(relative);
        continue;
      }
      if (!entry.isFile() || !relative.endsWith(".html") ||
          ["404.html", "print.html", "toc.html"].includes(relative)) continue;
      const html = await readFile(path.join(book, relative), "utf8");
      if (/http-equiv\s*=\s*["']refresh["']/i.test(html)) continue;
      const main = html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/i);
      if (!main) throw new Error(`No main content in ${relative}`);
      // Remove heading permalinks so they do not clutter the Markdown headings.
      const content = main[1].replace(/<a\b[^>]*class="header"[^>]*>([\s\S]*?)<\/a>/gi, "$1");
      const markdown = execFileSync("pandoc", ["--from=html", "--to=gfm-raw_html", "--wrap=none"], {
        input: content, encoding: "utf8",
      });
      const destination = path.join(book, "_markdown", relative.replace(/\.html$/, ".md"));
      await mkdir(path.dirname(destination), { recursive: true });
      await writeFile(destination, markdown);
    }
  }
  await visit();
}
