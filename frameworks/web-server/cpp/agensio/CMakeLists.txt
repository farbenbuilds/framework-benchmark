cmake_minimum_required(VERSION 3.20)
project(agensio_benchmark CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(server main.cpp)
find_package(Threads REQUIRED)
target_link_libraries(server PRIVATE Threads::Threads)
