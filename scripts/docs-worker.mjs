function acceptsMarkdown(header) {
  return (header ?? "").split(",").some((part) => {
    const [type, ...parameters] = part.trim().split(";");
    if (type.toLowerCase() !== "text/markdown") return false;
    const quality = parameters.find((parameter) => /^\s*q\s*=/i.test(parameter));
    return !quality || Number(quality.split("=")[1]) > 0;
  });
}

function markdownPath(pathname) {
  if (pathname === "/") return "/_markdown/index.md";
  if (pathname.endsWith("/")) return `/_markdown${pathname}index.md`;
  if (pathname.endsWith(".html")) return `/_markdown${pathname.slice(0, -5)}.md`;
  return null;
}

function withVary(headers) {
  const vary = headers.get("vary");
  if (!vary?.split(",").some((name) => name.trim().toLowerCase() === "accept")) {
    headers.set("vary", vary ? `${vary}, Accept` : "Accept");
  }
  return headers;
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.hostname === "www.tomat.cc") {
      url.hostname = "tomat.cc";
      url.protocol = "https:";
      return Response.redirect(url, 308);
    }

    // mdBook links use .html, so directory indexes need explicit resolution
    // when Cloudflare's extension-removing HTML handling is disabled.
    if (url.pathname.endsWith("/")) url.pathname += "index.html";
    const response = await env.ASSETS.fetch(new Request(url, request));
    const page = markdownPath(url.pathname);
    if (page && ["GET", "HEAD"].includes(request.method)) {
      const htmlHeaders = withVary(new Headers(response.headers));
      if (acceptsMarkdown(request.headers.get("Accept")) && response.ok &&
          response.headers.get("Content-Type")?.toLowerCase().startsWith("text/html")) {
        const sourceUrl = new URL(page, url);
        const source = await env.ASSETS.fetch(new Request(sourceUrl, { method: request.method }));
        if (source.ok) {
          const headers = withVary(new Headers(source.headers));
          headers.set("Content-Type", "text/markdown; charset=utf-8");
          await response.body?.cancel();
          return new Response(source.body, { status: source.status, headers });
        }
        await source.body?.cancel();
      }
      return new Response(response.body, { status: response.status, headers: htmlHeaders });
    }
    if (url.pathname !== "/install" || !response.ok) return response;

    const headers = new Headers(response.headers);
    headers.set("Content-Type", "text/plain; charset=utf-8");
    return new Response(response.body, { status: response.status, headers });
  },
};
