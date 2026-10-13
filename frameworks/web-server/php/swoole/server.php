<?php

use Swoole\Http\Server;
use Swoole\Http\Request;
use Swoole\Http\Response;

const PLAINTEXT = "Hello, World!";
const JSON = '{"message":"Hello, World!"}';

$workers = filter_var(getenv('BENCHMARK_WORKERS'), FILTER_VALIDATE_INT, [
    'options' => ['min_range' => 1],
]) ?: swoole_cpu_num();
$http = new Server('0.0.0.0', 8080);
$http->set([
    'worker_num' => $workers,
    'open_tcp_nodelay' => true,
    'log_level' => SWOOLE_LOG_WARNING,
]);

$http->on('request', function (Request $request, Response $response): void {
    $uri = $request->server['request_uri'];

    if ($uri === '/healthz') {
        $response->status(200);
        $response->end();
        return;
    }

    if ($uri === '/plaintext') {
        $response->header('Content-Type', 'text/plain; charset=utf-8');
        $response->end(PLAINTEXT);
        return;
    }

    if ($uri === '/json') {
        $response->header('Content-Type', 'application/json');
        $response->end(JSON);
        return;
    }

    $response->status(404);
    $response->end();
});

$http->start();
