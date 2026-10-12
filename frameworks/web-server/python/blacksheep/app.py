from blacksheep import Application, json, text

app = Application()

@app.route("/healthz")
async def healthz():
    return text("")

@app.route("/plaintext")
async def plaintext():
    return text("Hello, World!")

@app.route("/json")
async def get_json():
    return json({"message": "Hello, World!"})
