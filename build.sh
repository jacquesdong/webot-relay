#!/bin/bash

# Build script for webot-relay using Ubuntu 20.04 Docker

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_IMAGE="webot-relay-builder"

echo "Building webot-relay with Ubuntu 20.04 Docker..."
echo ""

# Build Docker image if not exists or force rebuild
if [ "$1" = "--rebuild" ] || ! docker inspect "${DOCKER_IMAGE}" > /dev/null 2>&1; then
    echo "Building Docker image..."
    docker build -t "${DOCKER_IMAGE}" "${PROJECT_DIR}"
else
    echo "Using existing Docker image: ${DOCKER_IMAGE}"
fi

echo ""
echo "Starting build..."

# Run container with volume mount and build
docker run --rm \
    -v "${PROJECT_DIR}:/app" \
    -w /app \
    "${DOCKER_IMAGE}" \
    cargo build --release

echo ""
echo "Build completed successfully!"
echo "Binary location: ${PROJECT_DIR}/target/release/webot-relay"
echo ""
echo "You can verify the GLIBC version with:"
echo "  ldd ${PROJECT_DIR}/target/release/webot-relay"
echo ""
echo "Or check linked libraries:"
echo "  readelf -h ${PROJECT_DIR}/target/release/webot-relay | grep 'OS/ABI'"
