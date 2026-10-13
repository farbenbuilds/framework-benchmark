package main

import (
	"github.com/valyala/fasthttp"
)

var (
	plaintextBody = []byte("Hello, World!")
	jsonBody      = []byte("{\"message\": \"Hello, World!\"}")
)

func requestHandler(ctx *fasthttp.RequestCtx) {
	switch string(ctx.Path()) {
	case "/healthz":
		ctx.SetStatusCode(fasthttp.StatusOK)
	case "/plaintext":
		ctx.SetContentType("text/plain; charset=utf-8")
		ctx.SetBodyRaw(plaintextBody)
	case "/json":
		ctx.SetContentType("application/json")
		ctx.SetBodyRaw(jsonBody)
	default:
		ctx.Error("Not Found", fasthttp.StatusNotFound)
	}
}

func main() {
	_ = fasthttp.ListenAndServe(":8080", requestHandler)
}
