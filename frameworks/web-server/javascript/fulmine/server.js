import http from "node:http";

const plaintext = Buffer.from("Hello, World!");
const json = Buffer.from(JSON.stringify({ message: "Hello, World!" }));

const server = http.createServer((req, res) => {
  if (req.url === "/healthz") {
    res.writeHead(200);
    res.end();
  } else if (req.url === "/plaintext") {
    res.writeHead(200, {
      "Content-Type": "text/plain; charset=utf-8",
      "Content-Length": plaintext.length,
    });
    res.end(plaintext);
  } else if (req.url === "/json") {
    res.writeHead(200, {
      "Content-Type": "application/json",
      "Content-Length": json.length,
    });
    res.end(json);
  } else {
    res.writeHead(404);
    res.end();
  }
});

server.listen(8080);
