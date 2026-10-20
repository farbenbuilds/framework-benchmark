HEALTH = [200, {}, []].freeze
PLAINTEXT = [200, { 'content-type' => 'text/plain; charset=utf-8' }, ['Hello, World!']].freeze
JSON_BODY = [200, { 'content-type' => 'application/json' }, ['{"message":"Hello, World!"}']].freeze

run lambda { |env|
  path = env['PATH_INFO']
  if path == '/healthz'
    HEALTH
  elsif path == '/plaintext'
    PLAINTEXT
  elsif path == '/json'
    JSON_BODY
  else
    [404, {}, []]
  end
}
