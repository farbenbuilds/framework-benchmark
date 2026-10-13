<?php

require_once __DIR__ . '/vendor/autoload.php';

use Workerman\Worker;
use Workerman\Protocols\Http\Response;

const PLAINTEXT = "Hello, World!";
const JSON = '{"message":"Hello, World!"}';

$http_worker = new Worker("http://0.0.0.0:8080");
$http_worker->count = worker_count();
$http_worker->reusePort = true;

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
            PLAINTEXT
        ));
        return;
    }

    if ($path === '/json') {
        $connection->send(new Response(
            200,
            ['Content-Type' => 'application/json'],
            JSON
        ));
        return;
    }

    $connection->send(new Response(404));
};

Worker::runAll();

function worker_count(): int
{
    $configured = filter_var(getenv('BENCHMARK_WORKERS'), FILTER_VALIDATE_INT, [
        'options' => ['min_range' => 1],
    ]);
    return $configured ?: 1;
}
