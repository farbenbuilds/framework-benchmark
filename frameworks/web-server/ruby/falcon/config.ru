require 'json'

run lambda { |env|
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
}
