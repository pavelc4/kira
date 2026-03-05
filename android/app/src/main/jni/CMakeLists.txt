cmake_minimum_required(VERSION 3.22.1)

project("kiracore_jni")

add_library(
        kiracore_jni
        SHARED
        rust_jni.cpp)

find_library(
        log-lib
        log)

target_link_libraries(
        kiracore_jni
        ${log-lib})
