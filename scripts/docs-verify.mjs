import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const installer = await readFile(
  new URL("./tomat-installer.sh", import.meta.url),
  "utf8",
);
const base = process.argv[2] ?? "https://tomat.cc";

async function verify() {
  const home = await fetch(`${base}/`, {
    redirect: "manual",
    signal: AbortSignal.timeout(15000),
  });
  assert.equal(home.status, 200, "/: expected 200");
  const html = await home.text();
  assert.match(html, /Welcome to the documentation/);
  const searchIndex = html.match(
    /window\.path_to_searchindex_js = "([^"]+)"/,
  )[1];
  const mermaid = html.match(/src="(mermaid-[^"]+\.min\.js)"/)[1];
  for (const pathname of [
    "/guide/installation.html",
    "/configuration/",
    "/images/logo.svg",
    "/sitemap.xml",
    "/robots.txt",
    `/${searchIndex}`,
    `/${mermaid}`,
  ]) {
    const response = await fetch(`${base}${pathname}`, {
      redirect: "manual",
      signal: AbortSignal.timeout(15000),
    });
    assert.equal(
      response.status,
      200,
      `${pathname}: expected 200, got ${response.status}`,
    );
    await response.arrayBuffer();
  }

  for (const pathname of ["/", "/guide/installation.html", "/configuration/"]) {
    for (const method of ["GET", "HEAD"]) {
      const response = await fetch(`${base}${pathname}`, {
        method,
        headers: { Accept: "text/markdown" },
        signal: AbortSignal.timeout(15000),
      });
      assert.equal(response.status, 200, `${pathname}: expected Markdown page`);
      assert.match(response.headers.get("Content-Type"), /^text\/markdown/);
      assert.match(response.headers.get("Vary"), /(?:^|,\s*)Accept(?:,|$)/i);
      const body = await response.text();
      if (method === "HEAD") assert.equal(body, "");
      else {
        assert.match(body, /^# /m);
        assert.doesNotMatch(body, /<html|<nav|<script/i);
      }
    }
  }

  for (const method of ["GET", "HEAD"]) {
    const response = await fetch(`${base}/install`, {
      method,
      signal: AbortSignal.timeout(15000),
    });
    assert.equal(response.status, 200, `/install ${method}: expected 200`);
    assert.match(response.headers.get("Content-Type"), /^text\/plain/);
    assert.equal(await response.text(), method === "HEAD" ? "" : installer);
  }

  const missing = await fetch(`${base}/does-not-exist/`, {
    signal: AbortSignal.timeout(15000),
  });
  assert.equal(missing.status, 404, "Missing pages must return 404.");
  await missing.arrayBuffer();

  if (base === "https://tomat.cc") {
    const response = await fetch(
      "https://www.tomat.cc/guide/installation.html?q=a%26b",
      {
        redirect: "manual",
        signal: AbortSignal.timeout(15000),
      },
    );
    assert.equal(response.status, 308, "www must redirect permanently.");
    assert.equal(
      response.headers.get("Location"),
      "https://tomat.cc/guide/installation.html?q=a%26b",
    );
    await response.arrayBuffer();
  }
}

// Newly attached custom domains may take a few minutes to resolve worldwide.
for (let attempt = 1; ; attempt++) {
  try {
    await verify();
    console.log(
      `Verified documentation, Markdown negotiation, assets, installer, and routing at ${base}.`,
    );
    break;
  } catch (error) {
    if (attempt === 12) throw error;
    console.warn(
      `Verification attempt ${attempt} failed: ${error.message}. Retrying in 10 seconds.`,
    );
    await new Promise((resolve) => setTimeout(resolve, 10000));
  }
}
