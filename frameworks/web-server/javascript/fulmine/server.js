import express from "express";

const app = express({ cluster: "auto" });

app.get("/healthz", (_, response) => response.status(200).end());
app.get("/plaintext", (_, response) =>
  response.type("text/plain").send("Hello, World!"),
);
app.get("/json", (_, response) =>
  response.type("application/json").send('{"message":"Hello, World!"}'),
);

app.listen(8080);
