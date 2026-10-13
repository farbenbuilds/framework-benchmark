import io.vertx.core.Vertx;
import io.vertx.core.buffer.Buffer;
import io.vertx.core.http.HttpServer;
import io.vertx.ext.web.Router;

public class Main {
    private static final Buffer PLAINTEXT = Buffer.buffer("Hello, World!");
    private static final Buffer JSON = Buffer.buffer("{\"message\":\"Hello, World!\"}");

    public static void main(String[] args) {
        Vertx vertx = Vertx.vertx();
        HttpServer server = vertx.createHttpServer();
        Router router = Router.router(vertx);

        router.get("/healthz").handler(ctx -> ctx.response().setStatusCode(200).end());

        router.get("/plaintext").handler(ctx -> ctx.response()
            .putHeader("Content-Type", "text/plain; charset=utf-8")
            .end(PLAINTEXT));

        router.get("/json").handler(ctx -> ctx.response()
            .putHeader("Content-Type", "application/json")
            .end(JSON));

        server.requestHandler(router).listen(8080);
    }
}
