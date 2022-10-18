set(ANDROID_NDK $ENV{ANDROID_NDK})
set(ANDROID_ABI "x86")
set(ANDROID_PLATFORM "android-19")

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
# allow cmake to search outside of NDK sysroot
# https://gitlab.kitware.com/cmake/cmake/-/issues/22183
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY BOTH)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE BOTH)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE BOTH)

include(${ANDROID_NDK}/build/cmake/android.toolchain.cmake)
