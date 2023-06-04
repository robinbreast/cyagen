cmake_minimum_required(VERSION 3.14)
project(test_sample)

# GoogleTest requires at least C++14
set(CMAKE_CXX_STANDARD 14)
cmake_policy(SET CMP0135 NEW)

include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

enable_testing()
# MANUAL SECTION: 96a20eef-fc0b-5970-878d-078b541544bd
# MANUAL SECTION END
include_directories(
  ../../../source
# MANUAL SECTION: cc2fef65-c36a-59e8-a174-13973e3c3ed9
# MANUAL SECTION END
)
add_executable(
    test_sample
    test_sample.cc
# MANUAL SECTION: 63a7443f-08ba-5ae7-b18d-b5722c8496cc
# MANUAL SECTION END
)
target_link_libraries(
    test_sample
    GTest::gtest_main
    GTest::gmock_main
# MANUAL SECTION: 16d351ce-64c8-57d3-ac70-efce6455dee8
# MANUAL SECTION END
)
include(GoogleTest)
gtest_discover_tests(test_sample)
