<?php

$uri = $_SERVER['REQUEST_URI'] ?? '/';

if ($uri === '/healthz') {
    http_response_code(200);
    exit;
}

if ($uri === '/plaintext') {
    header("Content-Type: text/plain; charset=utf-8");
    echo "Hello, World!";
    exit;
}

if ($uri === '/json') {
    header("Content-Type: application/json");
    echo json_encode(["message" => "Hello, World!"]);
    exit;
}

http_response_code(404);
