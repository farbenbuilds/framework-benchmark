#include "core/HttpServer.h"

using namespace hical;

int main() {
  HttpServer server(8080);
  server.setIdleTimeout(0);

  server.router().get("/healthz", [](const HttpRequest &) -> HttpResponse {
    return HttpResponse::ok();
  });
  server.router().get("/plaintext", [](const HttpRequest &) -> HttpResponse {
    return HttpResponse::ok("Hello, World!", "text/plain; charset=utf-8");
  });
  server.router().get("/json", [](const HttpRequest &) -> HttpResponse {
    return HttpResponse::ok("{\"message\": \"Hello, World!\"}",
                            "application/json");
  });

  server.start();
}
