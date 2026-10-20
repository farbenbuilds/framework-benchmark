const std = @import("std");
const http = @import("dusty");

fn health(_: *http.Request, res: *http.Response) !void {
    res.status = .ok;
    res.body = "";
}

fn plaintext(_: *http.Request, res: *http.Response) !void {
    res.status = .ok;
    res.content_type = .text;
    res.body = "Hello, World!";
}

fn json(_: *http.Request, res: *http.Response) !void {
    res.status = .ok;
    res.content_type = .json;
    res.body = "{\"message\":\"Hello, World!\"}";
}

pub fn main(init: std.process.Init) !void {
    const address: http.Address = .{
        .ip = try std.Io.net.IpAddress.parse("0.0.0.0", 8080),
    };
    const listeners = [_]http.Listener{.{ .address = address }};
    var server = http.Server(void).init(init.gpa, init.io, .{ .listeners = &listeners }, {});
    defer server.deinit();

    server.router.get("/healthz", health);
    server.router.get("/plaintext", plaintext);
    server.router.get("/json", json);

    try server.run();
}
