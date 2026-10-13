#!/usr/bin/env sh
set -eu

apply=0
push=0

for arg in "$@"; do
    case $arg in
        --apply) apply=1 ;;
        --push) push=1 ;;
        *) echo "usage: commit_benchmark_results.sh [--apply] [--push]" >&2; exit 2 ;;
    esac
done

[ "$push" -eq 0 ] || [ "$apply" -eq 1 ] || {
    echo '--push requires --apply.' >&2
    exit 2
}

git diff --check -- results README.md
git status --short -- results README.md
if [ "$apply" -eq 0 ]; then
    echo 'Dry run. Re-run with --apply after reviewing this result diff.'
    exit 0
fi

git add -- results README.md
git diff --cached --check
git diff --cached --quiet && {
    echo 'No generated results to commit.' >&2
    exit 1
}
git commit -m 'chore(benchmark): update results'
[ "$push" -eq 0 ] || git push
