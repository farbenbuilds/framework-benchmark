require 'agoo'
require 'etc'

HEALTH = [200, {}, []].freeze
PLAINTEXT = [200, { 'Content-Type' => 'text/plain; charset=utf-8' }, ['Hello, World!']].freeze
JSON = [200, { 'Content-Type' => 'application/json' }, ['{"message":"Hello, World!"}']].freeze
THREADS = Integer(ENV.fetch('BENCHMARK_THREADS', Etc.nprocessors.to_s), exception: false) || 1

Agoo::Server.init(8080, '.', thread_count: [THREADS, 1].max)
Agoo::Server.handle(:GET, '/healthz', ->(_) { HEALTH })
Agoo::Server.handle(:GET, '/plaintext', ->(_) { PLAINTEXT })
Agoo::Server.handle(:GET, '/json', ->(_) { JSON })
Agoo::Server.start
sleep
