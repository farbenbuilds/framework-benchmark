using System.Text;
using Sisk.Cadente.CoreEngine;
using Sisk.Core.Http;

using var app = HttpServer.CreateBuilder()
    .UseEngine(new CadenteHttpServerEngine())
    .UseListeningPort("http://0.0.0.0:8080/")
    .UseMinimalConfiguration()
    .UseRouter(router =>
    {
        router.MapGet("/healthz", _ => new HttpResponse());
        router.MapGet("/plaintext", _ => Response("Hello, World!", "text/plain"));
        router.MapGet("/json", _ => Response("{\"message\":\"Hello, World!\"}", "application/json"));
    })
    .Build();

app.Start(verbose: false);

static HttpResponse Response(string body, string content_type) => new()
{
    Content = new StringContent(body, Encoding.UTF8, content_type),
};
