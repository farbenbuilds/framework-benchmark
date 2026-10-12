require 'iodine'
require 'json'

app = proc do |env|
  path = env['PATH_INFO']
  if path == '/healthz'
    [200, {}, []]
  elsif path == '/plaintext'
    [200, { 'content-type' => 'text/plain; charset=utf-8' }, ['Hello, World!']]
  elsif path == '/json'
    [200, { 'content-type' => 'application/json' }, [JSON.generate({ message: 'Hello, World!' })]]
  else
    [404, {}, []]
  end
end

Iodine.listen(service: :http, port: 8080, handler: app)
Iodine.threads = 1
Iodine.workers = 1
Iodine.start
