#!/usr/bin/env sh
set -eu

usage() {
    echo "usage: build_container.sh <context_dir> <tag> [glibc|musl]" >&2
}

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    usage
    exit 2
fi

context_dir=$1
tag=$2
target_libc=${3:-glibc}
runtime=${CONTAINER_RUNTIME:-docker}
pull=${BENCHMARK_IMAGE_PULL:-0}
no_cache=${BENCHMARK_NO_CACHE:-0}

case $target_libc in
    glibc|musl) ;;
    *) echo "libc must be glibc or musl" >&2; exit 2 ;;
esac
case $pull:$no_cache in
    0:0|0:1|1:0|1:1) ;;
    *) echo "BENCHMARK_IMAGE_PULL and BENCHMARK_NO_CACHE must be 0 or 1" >&2; exit 2 ;;
esac

dockerfile=$context_dir/Dockerfile.$target_libc
if [ ! -d "$context_dir" ] || [ ! -f "$dockerfile" ]; then
    echo "Dockerfile not found: $dockerfile" >&2
    exit 1
fi
if ! command -v "$runtime" >/dev/null 2>&1; then
    echo "Container runtime not found: $runtime" >&2
    exit 1
fi

set -- build --file "$dockerfile" --tag "$tag"
[ "$pull" = 1 ] && set -- "$@" --pull
[ "$no_cache" = 1 ] && set -- "$@" --no-cache
"$runtime" "$@" "$context_dir"

echo "Built $tag ($target_libc)."
