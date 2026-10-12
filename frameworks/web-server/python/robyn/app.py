from robyn import Robyn, jsonify

app = Robyn(__file__)

@app.get("/healthz")
async def healthz():
    return ""

@app.get("/plaintext")
async def plaintext():
    return "Hello, World!"

@app.get("/json")
async def get_json():
    return jsonify({"message": "Hello, World!"})

if __name__ == "__main__":
    app.start(port=8080, host="0.0.0.0")
