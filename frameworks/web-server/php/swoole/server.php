<?php

use Swoole\Http\Server;
use Swoole\Http\Request;
use Swoole\Http\Response;

$http = new Server("0.0.0.0", 8080);

$http->on("request", function (Request $request, Response $response) {
    $uri = $request->server['request_uri'];

    if ($uri === '/healthz') {
        $response->status(200);
        $response->end();
        return;
    }

    if ($uri === '/plaintext') {
        $response->header("Content-Type", "text/plain; charset=utf-8");
        $response->end("Hello, World!");
        return;
    }

    if ($uri === '/json') {
        $response->header("Content-Type", "application/json");
        $response->end(json_encode(["message" => "Hello, World!"]));
        return;
    }

    $response->status(404);
    $response->end();
});

$http->start();
