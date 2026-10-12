#include <drogon/drogon.h>

int main()
{
    drogon::app()
        .registerHandler("/healthz", [](const drogon::HttpRequestPtr &, auto &&callback) {
            callback(drogon::HttpResponse::newHttpResponse());
        })
        .registerHandler("/plaintext", [](const drogon::HttpRequestPtr &, auto &&callback) {
            auto response = drogon::HttpResponse::newHttpResponse();
            response->setContentTypeCode(drogon::CT_TEXT_PLAIN);
            response->setBody("Hello, World!");
            callback(response);
        })
        .registerHandler("/json", [](const drogon::HttpRequestPtr &, auto &&callback) {
            Json::Value body;
            body["message"] = "Hello, World!";
            callback(drogon::HttpResponse::newHttpJsonResponse(body));
        })
        .addListener("0.0.0.0", 8080)
        .setThreadNum(0)
        .run();
}
