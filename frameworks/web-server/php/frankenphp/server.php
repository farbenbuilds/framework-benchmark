<?php

$server = stream_socket_server("tcp://0.0.0.0:8080", $errno, $errstr);
if (!$server) {
    exit(1);
}

$json = json_encode(['message' => 'Hello, World!']);

while ($conn = stream_socket_accept($server)) {
    $req = fread($conn, 1024);
    if (strpos($req, 'GET /healthz') === 0) {
        fwrite($conn, "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    } elseif (strpos($req, 'GET /plaintext') === 0) {
        fwrite($conn, "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 13\r\n\r\nHello, World!");
    } elseif (strpos($req, 'GET /json') === 0) {
        $len = strlen($json);
        fwrite($conn, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {$len}\r\n\r\n{$json}");
    } else {
        fwrite($conn, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
    }
    fclose($conn);
}
