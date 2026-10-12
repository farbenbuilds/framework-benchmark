import org.rapidoid.setup.On;

import java.util.Collections;

public class Main {
    public static void main(String[] args) {
        On.port(8080);
        On.get("/healthz").plain(() -> "");
        On.get("/plaintext").plain(() -> "Hello, World!");
        On.get("/json").json(() -> Collections.singletonMap("message", "Hello, World!"));
    }
}
