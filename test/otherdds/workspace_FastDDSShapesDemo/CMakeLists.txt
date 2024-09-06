cmake_minimum_required(VERSION 3.20)

project(FastDDSShapesDemo)

# Find requirements
if(NOT fastcdr_FOUND)
    find_package(fastcdr 2 REQUIRED)
endif()

if(NOT fastdds_FOUND)
    find_package(fastdds 3 REQUIRED)
endif()

# Set C++11
include(CheckCXXCompilerFlag)
if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_COMPILER_IS_CLANG OR
        CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    check_cxx_compiler_flag(-std=c++11 SUPPORTS_CXX11)
    if(SUPPORTS_CXX11)
        add_compile_options(-std=c++11)
    else()
        message(FATAL_ERROR "Compiler doesn't support C++11")
    endif()
endif()

message(STATUS "Configuring ShapesDemo publisher/subscriber example...")
file(GLOB DDS_HELLOWORLD_SOURCES_CXX "src/*.cxx")

add_executable(FastDDSShapesDemoPublisher src/ShapesDemoPublisher.cpp ${DDS_HELLOWORLD_SOURCES_CXX})
target_link_libraries(FastDDSShapesDemoPublisher fastdds fastcdr)

add_executable(FastDDSShapesDemoSubscriber src/ShapesDemoSubscriber.cpp ${DDS_HELLOWORLD_SOURCES_CXX})
target_link_libraries(FastDDSShapesDemoSubscriber fastdds fastcdr)

