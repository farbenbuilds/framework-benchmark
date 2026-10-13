use cpeak::{Cpeak, CpeakError, CpeakOptions, CpeakRequest, CpeakResponse};

fn main() {
    let mut server = Cpeak::new(CpeakOptions::default());
    server.route("GET", "/healthz", healthz);
    server.route("GET", "/plaintext", plaintext);
    server.route("GET", "/json", json);
    server.listen("0.0.0.0:8080", || {}).wait();
}

async fn healthz(_: &mut CpeakRequest, response: &mut CpeakResponse) -> Result<(), CpeakError> {
    response.status(200).end("")
}

async fn plaintext(_: &mut CpeakRequest, response: &mut CpeakResponse) -> Result<(), CpeakError> {
    response
        .status(200)
        .set_header("content-type", "text/plain; charset=utf-8")
        .end("Hello, World!")
}

async fn json(_: &mut CpeakRequest, response: &mut CpeakResponse) -> Result<(), CpeakError> {
    response
        .status(200)
        .set_header("content-type", "application/json")
        .end("{\"message\":\"Hello, World!\"}")
}
