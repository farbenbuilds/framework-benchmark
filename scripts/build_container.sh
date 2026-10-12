#!/usr/bin/env sh
set -eu

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <context_dir> <tag> [target_libc]" >&2
    exit 1
fi

CONTEXT_DIR="$1"
TAG="$2"
TARGET_LIBC="${3:-glibc}"
CONTAINER_RUNTIME="${CONTAINER_RUNTIME:-docker}"

DOCKERFILE="${CONTEXT_DIR}/Dockerfile.${TARGET_LIBC}"

if [ ! -f "$DOCKERFILE" ]; then
    echo "Dockerfile not found: $DOCKERFILE" >&2
    exit 1
fi

if ! command -v "$CONTAINER_RUNTIME" >/dev/null 2>&1; then
    echo "Container runtime not found: $CONTAINER_RUNTIME" >&2
    exit 1
fi

echo "Building container image: ${TAG} (${TARGET_LIBC}) without cache drift..."

"$CONTAINER_RUNTIME" build \
    --no-cache \
    --pull \
    --tag "${TAG}" \
    --file "${DOCKERFILE}" \
    "${CONTEXT_DIR}"

echo "Build complete: ${TAG}"
