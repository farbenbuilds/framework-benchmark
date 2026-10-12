#include <iostream>
#include <string>
#include <cstring>
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>

int main() {
    int server_fd = socket(AF_INET, SOCK_STREAM, 0);
    int opt = 1;
    setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    sockaddr_in address{};
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = INADDR_ANY;
    address.sin_port = htons(8080);

    bind(server_fd, (struct sockaddr*)&address, sizeof(address));
    listen(server_fd, 4096);

    while (true) {
        int client_fd = accept(server_fd, nullptr, nullptr);
        if (client_fd < 0) continue;

        char buffer[1024] = {0};
        read(client_fd, buffer, sizeof(buffer) - 1);

        if (strstr(buffer, "GET /healthz") == buffer) {
            const char* resp = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
            write(client_fd, resp, strlen(resp));
        } else if (strstr(buffer, "GET /plaintext") == buffer) {
            const char* resp = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 13\r\n\r\nHello, World!";
            write(client_fd, resp, strlen(resp));
        } else if (strstr(buffer, "GET /json") == buffer) {
            const char* resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n{\"message\": \"Hello, World!\"}";
            write(client_fd, resp, strlen(resp));
        } else {
            const char* resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            write(client_fd, resp, strlen(resp));
        }
        close(client_fd);
    }
    return 0;
}
