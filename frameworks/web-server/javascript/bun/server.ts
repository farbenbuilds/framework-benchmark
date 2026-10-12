const server = Bun.serve({
  port: 8080,
  fetch(req) {
    const url = new URL(req.url);
    if (url.pathname === "/healthz") {
      return new Response(null, { status: 200 });
    }
    if (url.pathname === "/plaintext") {
      return new Response("Hello, World!", {
        headers: { "Content-Type": "text/plain; charset=utf-8" },
      });
    }
    if (url.pathname === "/json") {
      return Response.json({ message: "Hello, World!" });
    }
    return new Response("Not Found", { status: 404 });
  },
});
