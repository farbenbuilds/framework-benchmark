import { Elysia } from "elysia";

new Elysia()
  .get("/healthz", () => new Response(null, { status: 200 }))
  .get("/plaintext", () => new Response("Hello, World!", {
    headers: { "Content-Type": "text/plain; charset=utf-8" },
  }))
  .get("/json", () => ({ message: "Hello, World!" }))
  .listen(8080);
