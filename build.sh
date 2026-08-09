#!/bin/bash

set -e

BUILD_DIR="cmake-build-debug"
BUILD_TYPE="Debug"
VCPKG_TOOLCHAIN="$PWD/vcpkg/scripts/buildsystems/vcpkg.cmake"
COMPILER_C="/opt/homebrew/opt/llvm/bin/clang"
COMPILER_CXX="/opt/homebrew/opt/llvm/bin/clang++"

CORES=$(sysctl -n hw.ncpu)

if [ "$1" == "help" ]; then
    echo "Usage: ./build.sh [command]"
    echo "Commands:"
    echo "  (none)  : Configure and build the project"
    echo "  clean   : Remove the build directory and reset"
    echo "  test    : Run tests after building"
    exit 0
fi

if [ "$1" == "clean" ]; then
    echo "Cleaning build directory..."
    rm -rf "$BUILD_DIR"
    echo "Clean complete!"
    exit 0
fi

echo "Configuring project (Build Type: $BUILD_TYPE)..."
cmake -B "$BUILD_DIR" \
      -G Ninja \
      -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
      -DCMAKE_TOOLCHAIN_FILE="$VCPKG_TOOLCHAIN" \
      -DCMAKE_C_COMPILER="$COMPILER_C" \
      -DCMAKE_CXX_COMPILER="$COMPILER_CXX" \
      -DCMAKE_EXPORT_COMPILE_COMMANDS=ON

echo "Building project with $CORES cores..."
cmake --build "$BUILD_DIR" -j "$CORES"

if [ "$1" == "test" ]; then
    echo "Running tests..."
    cd "$BUILD_DIR"
    ctest --output-on-failure
    cd ..
fi

echo "All done!"
