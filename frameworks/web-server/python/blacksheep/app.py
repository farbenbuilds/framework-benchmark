from blacksheep import Application, Content, Response

app = Application()
PLAINTEXT = Content(b"text/plain", b"Hello, World!")
JSON = Content(b"application/json", b'{"message":"Hello, World!"}')

@app.route("/healthz")
async def healthz() -> Response:
    return Response(200)

@app.route("/plaintext")
async def plaintext() -> Response:
    return Response(200, content=PLAINTEXT)

@app.route("/json")
async def get_json() -> Response:
    return Response(200, content=JSON)
