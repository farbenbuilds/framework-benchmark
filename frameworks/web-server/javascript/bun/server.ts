Bun.serve({
  port: 8080,
  routes: {
    "/healthz": new Response(null, { status: 200 }),
    "/plaintext": new Response("Hello, World!", {
      headers: { "Content-Type": "text/plain; charset=utf-8" },
    }),
    "/json": new Response('{"message":"Hello, World!"}', {
      headers: { "Content-Type": "application/json" },
    }),
  },
  fetch: () => new Response(null, { status: 404 }),
});
