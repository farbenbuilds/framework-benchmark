const std = @import("std");
const swerver = @import("swerver");

fn health(ctx: *swerver.router.HandlerContext) swerver.response.Response {
    return ctx.text(200, "");
}

fn plaintext(ctx: *swerver.router.HandlerContext) swerver.response.Response {
    return ctx.text(200, "Hello, World!");
}

fn json(ctx: *swerver.router.HandlerContext) swerver.response.Response {
    return ctx.jsonValue(200, .{ .message = "Hello, World!" });
}

pub fn main() !void {
    var gpa: std.heap.DebugAllocator(.{}) = .init;
    defer _ = gpa.deinit();

    var router = swerver.router.Router.init(.{});
    try router.get("/healthz", health);
    try router.get("/plaintext", plaintext);
    try router.get("/json", json);

    const server = try swerver.ServerBuilder
        .configDefault()
        .router(router)
        .build(gpa.allocator());
    defer {
        server.deinit();
        gpa.allocator().destroy(server);
    }
    try server.run(null);
}
