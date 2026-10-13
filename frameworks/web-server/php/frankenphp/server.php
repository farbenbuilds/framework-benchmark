<?php

const PLAINTEXT = "Hello, World!";
const JSON = '{"message":"Hello, World!"}';

$handler = static function (): void {
    $uri = $_SERVER['REQUEST_URI'] ?? '/';

    if ($uri === '/healthz') {
        http_response_code(200);
        return;
    }

    if ($uri === '/plaintext') {
        header('Content-Type: text/plain; charset=utf-8');
        echo PLAINTEXT;
        return;
    }

    if ($uri === '/json') {
        header('Content-Type: application/json');
        echo JSON;
        return;
    }

    http_response_code(404);
};

while (frankenphp_handle_request($handler)) {
}
