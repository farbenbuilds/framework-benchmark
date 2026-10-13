#include "core/HttpServer.h"

#include <boost/json.hpp>

using namespace hical;

int main()
{
	HttpServer server(8080);
	server.setIdleTimeout(0);

	server.router().get("/healthz", [](const HttpRequest&) -> HttpResponse {
		return HttpResponse::ok();
	});
	server.router().get("/plaintext", [](const HttpRequest&) -> HttpResponse {
		return HttpResponse::ok("Hello, World!", "text/plain; charset=utf-8");
	});
	server.router().get("/json", [](const HttpRequest&) -> HttpResponse {
		return HttpResponse::json(boost::json::object {
			{"message", "Hello, World!"},
		});
	});

	server.start();
}
