var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

app.MapGet("/healthz", () => Results.Ok());
app.MapGet("/plaintext", () => Results.Text("Hello, World!", "text/plain; charset=utf-8"));
app.MapGet("/json", () => Results.Json(new { message = "Hello, World!" }));

app.Run("http://0.0.0.0:8080");
