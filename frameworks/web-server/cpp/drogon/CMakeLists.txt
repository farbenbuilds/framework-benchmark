cmake_minimum_required(VERSION 3.20)
project(drogon_benchmark CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Drogon CONFIG REQUIRED)

add_executable(server main.cpp)
target_link_libraries(server PRIVATE Drogon::Drogon)
