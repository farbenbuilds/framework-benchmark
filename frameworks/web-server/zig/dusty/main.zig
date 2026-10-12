const std = @import("std");
const net = std.net;

pub fn main() !void {
    const address = try net.Address.parseIp4("0.0.0.0", 8080);
    var server = try address.listen(.{ .reuse_address = true });
    defer server.deinit();

    while (true) {
        const connection = try server.accept();
        try handleConnection(connection);
    }
}

fn handleConnection(conn: net.Server.Connection) !void {
    defer conn.stream.close();
    var buf: [1024]u8 = undefined;
    const bytes_read = try conn.stream.read(&buf);
    if (bytes_read == 0) return;

    const req = buf[0..bytes_read];
    if (std.mem.startsWith(u8, req, "GET /healthz")) {
        _ = try conn.stream.write("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    } else if (std.mem.startsWith(u8, req, "GET /plaintext")) {
        _ = try conn.stream.write("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 13\r\n\r\nHello, World!");
    } else if (std.mem.startsWith(u8, req, "GET /json")) {
        const json = "{\"message\": \"Hello, World!\"}";
        var header_buf: [128]u8 = undefined;
        const resp = try std.fmt.bufPrint(&header_buf, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {d}\r\n\r\n{s}", .{ json.len, json });
        _ = try conn.stream.write(resp);
    } else {
        _ = try conn.stream.write("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
    }
}
