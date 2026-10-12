import io.undertow.Undertow;
import io.undertow.server.HttpHandler;
import io.undertow.server.HttpServerExchange;
import io.undertow.util.Headers;
import io.undertow.util.HttpString;

import java.nio.ByteBuffer;

public class Main {
    public static void main(String[] args) {
        Undertow server = Undertow.builder()
            .addHttpListener(8080, "0.0.0.0")
            .setHandler(new HttpHandler() {
                @Override
                public void handleRequest(HttpServerExchange exchange) {
                    String path = exchange.getRequestPath();
                    if (path.equals("/healthz")) {
                        exchange.setStatusCode(200);
                        exchange.endExchange();
                    } else if (path.equals("/plaintext")) {
                        exchange.getResponseHeaders().put(Headers.CONTENT_TYPE, "text/plain; charset=utf-8");
                        exchange.getResponseSender().send("Hello, World!");
                    } else if (path.equals("/json")) {
                        exchange.getResponseHeaders().put(Headers.CONTENT_TYPE, "application/json");
                        exchange.getResponseSender().send("{\"message\": \"Hello, World!\"}");
                    } else {
                        exchange.setStatusCode(404);
                        exchange.endExchange();
                    }
                }
            })
            .build();
        server.start();
    }
}
