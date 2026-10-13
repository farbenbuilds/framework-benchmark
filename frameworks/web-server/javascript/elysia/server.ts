import { Elysia } from "elysia";

new Elysia({ aot: true, nativeStaticResponse: true })
  .get("/healthz", "")
  .get("/plaintext", "Hello, World!")
  .get("/json", { message: "Hello, World!" })
  .listen(8080);
