using GenHTTP.Engine.Internal;
using GenHTTP.Modules.IO;
using GenHTTP.Modules.Layouting;

var app = Layout.Create()
    .Add("healthz", Content.From(Resource.FromString("")))
    .Add("plaintext", Content.From(Resource.FromString("Hello, World!")))
    .Add("json", Content.From(Resource.FromString("{\"message\": \"Hello, World!\"}")));

var host = Host.Create()
    .Handler(app)
    .Port(8080);

return await host.RunAsync();
