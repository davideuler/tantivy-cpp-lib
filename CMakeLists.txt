if (POLICY CMP0048)
  cmake_policy(SET CMP0048 NEW)
endif ()

project(tantivy_indexer_lib)

cmake_minimum_required(VERSION 3.15)

set (CMAKE_CXX_STANDARD 17)

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_CMD cargo build --verbose)
    set(TARGET_DIR "debug")
else ()
    set(CARGO_CMD cargo build --release --verbose)
    set(TARGET_DIR "release")
endif ()

if(ENABLE_LTO)
    set(RUST_FLAGS "-Clinker-plugin-lto" "-Clinker=clang-13" "-Clink-arg=-fuse-ld=lld-13")
endif()

set(TANTIVY_INDEXER_LIB "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/libtantivy_indexer_lib.a")

set(RUST_PART_CXX "${CMAKE_CURRENT_BINARY_DIR}/tantivy_indexer.cpp")
add_library(tantivy_indexer STATIC ${RUST_PART_CXX})
add_custom_command(
    OUTPUT ${RUST_PART_CXX}
    COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} RUSTFLAGS="${RUST_FLAGS}" ${CARGO_CMD}
    COMMAND cp ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/tantivy-indexer-lib/src/lib.rs.cc ${RUST_PART_CXX}
    COMMAND cp ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/tantivy-indexer-lib/src/lib.rs.h ${CMAKE_CURRENT_BINARY_DIR}/tantivy_indexer.h
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(tantivy_indexer pthread dl ${TANTIVY_INDEXER_LIB})

add_test(NAME tantivy_indexer_test 
    COMMAND cargo test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

## Building demo binary
include_directories(${CMAKE_BINARY_DIR}/cxxbridge/)
message(STATUS "CMake Binary Dir:  ${CMAKE_BINARY_DIR}")
include_directories(include)

add_executable(index_program_demo src/main.cpp)

target_link_libraries(index_program_demo ${TANTIVY_INDEXER_LIB})
