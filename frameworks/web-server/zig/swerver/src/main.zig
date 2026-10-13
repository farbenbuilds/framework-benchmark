const std = @import("std");
const swerver = @import("swerver");

fn health(ctx: *swerver.router.HandlerContext) swerver.response.Response {
    return ctx.text(200, "");
}

fn plaintext(ctx: *swerver.router.HandlerContext) swerver.response.Response {
    return ctx.text(200, "Hello, World!");
}

fn json(ctx: *swerver.router.HandlerContext) swerver.response.Response {
    return ctx.json(200, "{\"message\":\"Hello, World!\"}");
}

pub fn main() !void {
    var router = swerver.router.Router.init(.{});
    try router.get("/healthz", health);
    try router.get("/plaintext", plaintext);
    try router.get("/json", json);

    var config = swerver.config.ServerConfig.default();
    config.disable_middleware = true;
    config.disable_preencoded = false;
    const server = try swerver.ServerBuilder
        .config(config)
        .router(router)
        .build(std.heap.smp_allocator);
    defer {
        server.deinit();
        std.heap.smp_allocator.destroy(server);
    }
    try server.run(null);
}
