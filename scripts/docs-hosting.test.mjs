import assert from "node:assert/strict";
import {
  mkdtemp,
  mkdir,
  readFile,
  readdir,
  rm,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";
import { buildLegacyPages, redirectTarget } from "./docs-legacy.mjs";
import { buildSitemap } from "./docs-sitemap.mjs";
import worker from "./docs-worker.mjs";

test("sitemap includes content with directory URLs and excludes helper pages and redirects", async (t) => {
  const book = await mkdtemp(path.join(os.tmpdir(), "tomat-sitemap-"));
  t.after(() => rm(book, { recursive: true, force: true }));
  await mkdir(path.join(book, "guide"));
  for (const page of ["index.html", "404.html", "print.html", "toc.html", "guide/index.html", "guide/install & setup.html"]) {
    await writeFile(path.join(book, page), "<html>Documentation</html>");
  }
  await writeFile(path.join(book, "old.html"), '<meta http-equiv="refresh" content="0; url=guide/">');
  await writeFile(path.join(book, "logo.svg"), "<svg></svg>");
  await buildSitemap(book);
  const sitemap = await readFile(path.join(book, "sitemap.xml"), "utf8");
  assert.deepEqual([...sitemap.matchAll(/<loc>(.*?)<\/loc>/g)].map((match) => match[1]), [
    "https://tomat.cc/",
    "https://tomat.cc/guide/",
    "https://tomat.cc/guide/install%20%26%20setup.html",
  ]);
  await buildSitemap(book);
  assert.equal(await readFile(path.join(book, "sitemap.xml"), "utf8"), sitemap);
});

function assets(files) {
  const requests = [];
  return {
    requests,
    ASSETS: {
      async fetch(request) {
        requests.push(request);
        const file = files[new URL(request.url).pathname];
        return new Response(
          request.method === "HEAD" ? null : (file ?? "Missing page"),
          {
            status: file === undefined ? 404 : 200,
            headers: {
              "Content-Type": "application/octet-stream",
              ETag: '"fixture"',
            },
          },
        );
      },
    },
  };
}

test("www redirects paths and queries to HTTPS before asset lookup", async () => {
  const env = assets({});
  for (const method of ["GET", "HEAD"]) {
    for (const pathname of ["/", "/install", "/guide/installation.html"]) {
      const response = await worker.fetch(
        new Request(`http://www.tomat.cc${pathname}?q=a%26b`, { method }),
        env,
      );
      assert.equal(response.status, 308);
      assert.equal(
        response.headers.get("Location"),
        `https://tomat.cc${pathname}?q=a%26b`,
      );
    }
  }
  assert.equal(env.requests.length, 0);
});

test("directory indexes resolve without rewriting .html links", async () => {
  const env = assets({
    "/index.html": "Introduction",
    "/configuration/index.html": "Configuration",
    "/guide/installation.html": "Installation",
  });
  for (const [pathname, content] of [
    ["/", "Introduction"],
    ["/configuration/", "Configuration"],
    ["/guide/installation.html", "Installation"],
  ]) {
    const response = await worker.fetch(
      new Request(`https://tomat.cc${pathname}?q=1`),
      env,
    );
    assert.equal(response.status, 200);
    assert.equal(await response.text(), content);
    assert.equal(response.headers.get("Location"), null);
    assert.equal(new URL(env.requests.at(-1).url).search, "?q=1");
  }
});

test("missing pages retain the asset server's 404 status and body", async () => {
  const response = await worker.fetch(
    new Request("https://tomat.cc/missing/"),
    assets({}),
  );
  assert.equal(response.status, 404);
  assert.equal(await response.text(), "Missing page");
});

test("installer is plain text with unchanged bytes and supports HEAD", async () => {
  const script = "#!/bin/sh\nprintf 'tomat\\n'\n";
  const env = assets({ "/install": script });
  for (const method of ["GET", "HEAD"]) {
    const response = await worker.fetch(
      new Request("https://tomat.cc/install", { method }),
      env,
    );
    assert.equal(response.status, 200);
    assert.equal(
      response.headers.get("Content-Type"),
      "text/plain; charset=utf-8",
    );
    assert.equal(response.headers.get("ETag"), '"fixture"');
    assert.equal(await response.text(), method === "HEAD" ? "" : script);
    assert.equal(env.requests.at(-1).method, method);
  }
});

test("failed installer responses are not relabeled as shell scripts", async () => {
  const response = await worker.fetch(
    new Request("https://tomat.cc/install"),
    assets({}),
  );
  assert.equal(response.status, 404);
  assert.equal(
    response.headers.get("Content-Type"),
    "application/octet-stream",
  );
});

test("legacy targets preserve paths, escaped queries, and section anchors", () => {
  for (const [pathname, expected] of [
    ["/tomat", "/"],
    ["/tomat/", "/"],
    ["/tomat/index.html", "/index.html"],
    ["/tomat/configuration/", "/configuration/"],
    ["/tomat/guide/installation.html", "/guide/installation.html"],
    ["/tomat-other/page.html", "/tomat-other/page.html"],
    ["/tomat//example.org/", "//example.org/"],
  ]) {
    assert.equal(
      redirectTarget({ pathname, search: "?q=a%26b", hash: "#install-script" }),
      `https://tomat.cc${expected}?q=a%26b#install-script`,
    );
  }
});

test("generated Pages redirects work in the browser and retain the raw installer", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "tomat-pages-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const book = path.join(root, "book");
  const output = path.join(root, "pages");
  await mkdir(path.join(book, "guide"), { recursive: true });
  await mkdir(output);
  await writeFile(path.join(output, "stale.html"), "stale");
  for (const page of ["index.html", "404.html", "guide/installation.html"]) {
    await writeFile(path.join(book, page), "<html>Documentation</html>");
  }
  await writeFile(path.join(book, "image.svg"), "<svg></svg>");
  const installer = "#!/bin/sh\necho tomat\n";
  await writeFile(path.join(book, "install"), installer);
  await buildLegacyPages(book, output);

  assert.equal(await readFile(path.join(output, "install"), "utf8"), installer);
  assert.equal(
    await readFile(path.join(book, "guide/installation.html"), "utf8"),
    "<html>Documentation</html>",
  );
  assert.deepEqual((await readdir(output)).sort(), [
    ".nojekyll",
    "404.html",
    "guide",
    "index.html",
    "install",
  ]);

  for (const page of ["index.html", "404.html", "guide/installation.html"]) {
    const html = await readFile(path.join(output, page), "utf8");
    assert.match(html, /rel="canonical"/);
    assert.match(html, /http-equiv="refresh"/);
    assert.match(html, /<a href="https:\/\/tomat\.cc\//);
    let destination;
    vm.runInNewContext(html.match(/<script>([\s\S]*?)<\/script>/)[1], {
      URL,
      window: {
        location: {
          pathname:
            page === "404.html" ? "/tomat/missing.html" : `/tomat/${page}`,
          search: "?q=a%26b",
          hash: "#installation",
          replace(value) {
            destination = value;
          },
        },
      },
    });
    const expected = page === "404.html" ? "missing.html" : page;
    assert.equal(
      destination,
      `https://tomat.cc/${expected}?q=a%26b#installation`,
    );
  }
});
