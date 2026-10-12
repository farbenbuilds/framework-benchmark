require 'socket'
require 'json'

server = TCPServer.new('0.0.0.0', 8080)
json_body = JSON.generate({ message: 'Hello, World!' })

while (session = server.accept)
  request = session.gets
  if request.nil?
    session.close
    next
  end

  if request.start_with?('GET /healthz')
    session.print "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"
  elsif request.start_with?('GET /plaintext')
    session.print "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 13\r\n\r\nHello, World!"
  elsif request.start_with?('GET /json')
    session.print "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: #{json_body.bytesize}\r\n\r\n#{json_body}"
  else
    session.print "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n"
  end
  session.close
end
