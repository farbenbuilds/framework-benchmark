const zuws = @import("zuws");
const App = zuws.App;
const Request = zuws.Request;
const Response = zuws.Response;
const WebSocket = zuws.WebSocket;

pub fn main() !void {
    const app = try App.init();
    defer app.deinit();

    _ = app.get("/healthz", healthz);
    _ = app.get("/plaintext", plaintext);
    _ = app.get("/json", json);
    _ = app.ws("/ws", .{ .upgrade = upgrade, .message = echo });
    app.listen(8080, null);
    app.run();
}

fn healthz(res: *Response, _: *Request) callconv(.c) void {
    res.end("", false);
}

fn plaintext(res: *Response, _: *Request) callconv(.c) void {
    res.writeHeader("content-type", "text/plain; charset=utf-8");
    res.end("Hello, World!", false);
}

fn json(res: *Response, _: *Request) callconv(.c) void {
    res.writeHeader("content-type", "application/json");
    res.end("{\"message\":\"Hello, World!\"}", false);
}

fn upgrade(res: *Response, req: *Request, ctx: *zuws.c.SocketContext) callconv(.c) void {
    const key = req.getHeader("sec-websocket-key") orelse return bad_request(res);
    const protocol = req.getHeader("sec-websocket-protocol") orelse "";
    const extensions = req.getHeader("sec-websocket-extensions") orelse "";
    zuws.c.uws_res_upgrade(res, null, key.ptr, key.len, protocol.ptr, protocol.len, extensions.ptr, extensions.len, ctx);
}

fn bad_request(res: *Response) void {
    res.writeStatusCodeWithText(.BadRequest);
    res.endWithoutBody(true);
}

fn echo(ws: *WebSocket, message: []const u8, opcode: zuws.c.Opcode) void {
    _ = ws.send(message, opcode);
}
