import org.rapidoid.http.MediaType;
import org.rapidoid.http.Req;
import org.rapidoid.setup.On;

import java.nio.charset.StandardCharsets;

public class Main {
    private static final byte[] JSON = "{\"message\":\"Hello, World!\"}".getBytes(StandardCharsets.UTF_8);

    public static void main(String[] args) {
        On.port(8080);
        On.get("/healthz").plain(() -> "");
        On.get("/plaintext").plain(() -> "Hello, World!");
        On.get("/json").html((Req req) -> req.response().contentType(MediaType.JSON).body(JSON));
    }
}
