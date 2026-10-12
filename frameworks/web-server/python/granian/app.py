import json

PLAINTEXT_RESP = b"Hello, World!"
JSON_RESP = json.dumps({"message": "Hello, World!"}).encode("utf-8")

async def app(scope, receive, send):
    if scope["type"] != "http":
        return

    path = scope["path"]
    if path == "/healthz":
        await send({
            "type": "http.response.start",
            "status": 200,
            "headers": [[b"content-length", b"0"]],
        })
        await send({"type": "http.response.body", "body": b""})
    elif path == "/plaintext":
        await send({
            "type": "http.response.start",
            "status": 200,
            "headers": [
                [b"content-type", b"text/plain; charset=utf-8"],
                [b"content-length", str(len(PLAINTEXT_RESP)).encode("ascii")],
            ],
        })
        await send({"type": "http.response.body", "body": PLAINTEXT_RESP})
    elif path == "/json":
        await send({
            "type": "http.response.start",
            "status": 200,
            "headers": [
                [b"content-type", b"application/json"],
                [b"content-length", str(len(JSON_RESP)).encode("ascii")],
            ],
        })
        await send({"type": "http.response.body", "body": JSON_RESP})
    else:
        await send({
            "type": "http.response.start",
            "status": 404,
            "headers": [[b"content-length", b"0"]],
        })
        await send({"type": "http.response.body", "body": b""})
