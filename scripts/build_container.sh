#!/usr/bin/env sh
set -eu

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <context_dir> <tag> [target_libc]" >&2
    exit 1
fi

CONTEXT_DIR="$1"
TAG="$2"
TARGET_LIBC="${3:-glibc}"

DOCKERFILE="${CONTEXT_DIR}/Dockerfile.${TARGET_LIBC}"

if [ ! -f "$DOCKERFILE" ]; then
    echo "Dockerfile not found: $DOCKERFILE" >&2
    exit 1
fi

echo "Building Docker image: ${TAG} (${TARGET_LIBC}) without cache drift..."

docker build \
    --no-cache \
    --pull \
    --tag "${TAG}" \
    --file "${DOCKERFILE}" \
    "${CONTEXT_DIR}"

echo "Build complete: ${TAG}"
