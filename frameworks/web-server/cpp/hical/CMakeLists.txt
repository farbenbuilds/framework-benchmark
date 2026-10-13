cmake_minimum_required(VERSION 3.20)
project(hical_benchmark LANGUAGES C CXX)

set(HICAL_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(HICAL_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(HICAL_ENABLE_MEMORY_TRACKING OFF CACHE BOOL "" FORCE)
set(HICAL_WITH_OPENAPI OFF CACHE BOOL "" FORCE)
set(HICAL_DISABLE_IO_URING ON CACHE BOOL "" FORCE)

add_subdirectory(/opt/hical hical_build)
add_executable(server main.cpp)
target_link_libraries(server PRIVATE hical_core)
