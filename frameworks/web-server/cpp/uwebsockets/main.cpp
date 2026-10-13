#include <App.h>

#include <algorithm>
#include <cstdlib>
#include <thread>
#include <vector>

static void run_server() {
  uWS::App()
      .get("/healthz", [](auto *response, auto *) { response->end(); })
      .get("/plaintext",
           [](auto *response, auto *) {
             response->writeHeader("Content-Type", "text/plain; charset=utf-8")
                 ->end("Hello, World!");
           })
      .get("/json",
           [](auto *response, auto *) {
             response->writeHeader("Content-Type", "application/json")
                 ->end("{\"message\": \"Hello, World!\"}");
           })
      .listen(8080,
              [](auto *listen_socket) {
                if (!listen_socket)
                  std::exit(EXIT_FAILURE);
              })
      .run();
}

int main() {
  const auto thread_count = std::max(1u, std::thread::hardware_concurrency());
  std::vector<std::thread> workers;

  workers.reserve(thread_count);
  for (unsigned int index = 0; index < thread_count; ++index)
    workers.emplace_back(run_server);

  for (auto &worker : workers)
    worker.join();
}
