from robyn import Response, Robyn

app = Robyn(__file__)
PLAINTEXT = "Hello, World!"
JSON = '{"message":"Hello, World!"}'


@app.get("/healthz")
async def healthz():
    return ""


@app.get("/plaintext")
async def plaintext():
    return PLAINTEXT


@app.get("/json")
async def get_json():
    return Response(
        status_code=200,
        headers={"Content-Type": "application/json"},
        body=JSON,
    )


if __name__ == "__main__":
    app.start(port=8080, host="0.0.0.0")
