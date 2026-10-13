#include <drogon/drogon.h>

int main() {
  drogon::app()
      .registerHandler("/healthz",
                       [](const drogon::HttpRequestPtr &, auto &&callback) {
                         callback(drogon::HttpResponse::newHttpResponse());
                       })
      .registerHandler("/plaintext",
                       [](const drogon::HttpRequestPtr &, auto &&callback) {
                         auto response =
                             drogon::HttpResponse::newHttpResponse();
                         response->setContentTypeCode(drogon::CT_TEXT_PLAIN);
                         response->setBody("Hello, World!");
                         callback(response);
                       })
      .registerHandler(
          "/json",
          [](const drogon::HttpRequestPtr &, auto &&callback) {
            auto response = drogon::HttpResponse::newHttpResponse();
            response->setContentTypeCode(drogon::CT_APPLICATION_JSON);
            response->setBody("{\"message\":\"Hello, World!\"}");
            callback(response);
          })
      .addListener("0.0.0.0", 8080)
      .setThreadNum(0)
      .run();
}
