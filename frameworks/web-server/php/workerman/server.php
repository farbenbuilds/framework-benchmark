<?php

require_once __DIR__ . '/vendor/autoload.php';

use Workerman\Worker;
use Workerman\Protocols\Http\Response;

$http_worker = new Worker("http://0.0.0.0:8080");
$http_worker->count = 4;

$http_worker->onMessage = function ($connection, $request) {
    $path = $request->path();

    if ($path === '/healthz') {
        $connection->send(new Response(200));
        return;
    }

    if ($path === '/plaintext') {
        $connection->send(new Response(
            200,
            ['Content-Type' => 'text/plain; charset=utf-8'],
            'Hello, World!'
        ));
        return;
    }

    if ($path === '/json') {
        $connection->send(new Response(
            200,
            ['Content-Type' => 'application/json'],
            json_encode(['message' => 'Hello, World!'])
        ));
        return;
    }

    $connection->send(new Response(404));
};

Worker::runAll();
