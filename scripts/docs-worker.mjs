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
    if (url.pathname !== "/install" || !response.ok) return response;

    const headers = new Headers(response.headers);
    headers.set("Content-Type", "text/plain; charset=utf-8");
    return new Response(response.body, { status: response.status, headers });
  },
};
