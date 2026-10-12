cmake_minimum_required(VERSION 3.20)
project(uwebsockets_benchmark CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Threads REQUIRED)

add_library(usockets STATIC IMPORTED)
set_target_properties(usockets PROPERTIES
    IMPORTED_LOCATION /opt/uwebsockets/uSockets/uSockets.a
    INTERFACE_INCLUDE_DIRECTORIES /opt/uwebsockets/uSockets/src)

add_executable(server main.cpp)
target_include_directories(server PRIVATE /opt/uwebsockets/src)
target_compile_definitions(server PRIVATE LIBUS_NO_SSL UWS_NO_ZLIB)
target_link_libraries(server PRIVATE usockets Threads::Threads)
