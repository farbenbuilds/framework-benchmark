require 'agoo'
require 'json'

class Handler
  def call(env)
    path = env['PATH_INFO']
    if path == '/healthz'
      [200, {}, []]
    elsif path == '/plaintext'
      [200, { 'Content-Type' => 'text/plain; charset=utf-8' }, ['Hello, World!']]
    elsif path == '/json'
      [200, { 'Content-Type' => 'application/json' }, [JSON.generate({ message: 'Hello, World!' })]]
    else
      [404, {}, []]
    end
  end
end

Agoo::Server.init(8080, '.', thread_count: 1)
Agoo::Server.handle(:GET, '/healthz', Handler.new)
Agoo::Server.handle(:GET, '/plaintext', Handler.new)
Agoo::Server.handle(:GET, '/json', Handler.new)
Agoo::Server.start
