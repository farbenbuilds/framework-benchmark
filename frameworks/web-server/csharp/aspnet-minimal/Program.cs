var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();
const string Json = "{\"message\":\"Hello, World!\"}";

app.MapGet("/healthz", () => Results.Ok());
app.MapGet("/plaintext", () => Results.Text("Hello, World!", "text/plain; charset=utf-8"));
app.MapGet("/json", () => Results.Text(Json, "application/json"));

app.Run("http://0.0.0.0:8080");
