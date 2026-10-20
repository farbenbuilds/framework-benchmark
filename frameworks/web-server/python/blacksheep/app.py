from blacksheep import Application, Content, Response

app = Application()
PLAINTEXT = Content(b"text/plain", b"Hello, World!")
JSON = Content(b"application/json", b'{"message":"Hello, World!"}')


@app.router.get("/healthz")
async def healthz() -> Response:
    return Response(200)


@app.router.get("/plaintext")
async def plaintext() -> Response:
    return Response(200, content=PLAINTEXT)


@app.router.get("/json")
async def get_json() -> Response:
    return Response(200, content=JSON)
