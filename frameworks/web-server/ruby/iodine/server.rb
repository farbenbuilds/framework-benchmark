require 'iodine'
HEALTH = [200, {}, []].freeze
PLAINTEXT = [200, { 'content-type' => 'text/plain; charset=utf-8' }, ['Hello, World!']].freeze
JSON = [200, { 'content-type' => 'application/json' }, ['{"message":"Hello, World!"}']].freeze

app = proc do |env|
  path = env['PATH_INFO']
  if path == '/healthz'
    HEALTH
  elsif path == '/plaintext'
    PLAINTEXT
  elsif path == '/json'
    JSON
  else
    [404, {}, []]
  end
end

Iodine.listen(url: 'http://0.0.0.0:8080', handler: app)
Iodine.start
