#!/usr/bin/env sh
# Automated convention auditor for framework-benchmark.
# Verifies zero emojis, max 200 LOC per file, and marker slop ban.

set -eu

fail=0

echo "[1/4] Checking for forbidden emojis..."
# Unicode ranges covering emojis
emoji_files=$(git ls-files | grep -v '^\.agents/skills/' | while read -r file; do
    if [ -f "$file" ]; then
        if LC_ALL=C grep -P "[\x{1F300}-\x{1F9FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]" "$file" >/dev/null 2>&1; then
            echo "$file"
        fi
    fi
done)

if [ -n "$emoji_files" ]; then
    echo "ERROR: Emojis detected in the following files:"
    echo "$emoji_files"
    fail=1
else
    echo "PASS: Zero emojis found."
fi

echo "[2/4] Checking for files exceeding 200 lines..."
# Exclude lockfiles and vendored agent skills
long_files=$(git ls-files | grep -v '^\.agents/skills/' | grep -v 'skills-lock\.json$' | grep -v 'Cargo\.lock$' | while read -r file; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        if [ "$lines" -gt 200 ]; then
            echo "$file ($lines lines)"
        fi
    fi
done)

if [ -n "$long_files" ]; then
    echo "ERROR: Files exceeding 200 lines limit:"
    echo "$long_files"
    fail=1
else
    echo "PASS: All files under 200 lines limit."
fi

echo "[3/4] Checking for slop markers in source files..."
# Exclude auditor scripts and TODO.md from slop pattern search
slop_matches=$(git ls-files '*.rs' '*.sh' '*.toml' '*.yml' '*.yaml' | grep -v '^scripts/check_conventions\.' | while read -r file; do
    if [ -f "$file" ]; then
        if grep -nE '\b(FIXME|XXX|HACK)\b' "$file" >/dev/null 2>&1; then
            echo "$file"
        fi
    fi
done)

if [ -n "$slop_matches" ]; then
    echo "ERROR: Slop markers detected in:"
    echo "$slop_matches"
    fail=1
else
    echo "PASS: Zero slop markers found."
fi

echo "[4/4] Checking snake_case naming for docs and scripts..."
invalid_names=$(git ls-files 'docs/*' 'scripts/*' | while read -r file; do
    basename=$(basename "$file")
    case "$basename" in
        *[A-Z]*|*\ *)
            echo "$file"
            ;;
    esac
done)

if [ -n "$invalid_names" ]; then
    echo "ERROR: Non-snake_case filenames detected in docs or scripts:"
    echo "$invalid_names"
    fail=1
else
    echo "PASS: Filenames follow snake_case convention."
fi

if [ "$fail" -ne 0 ]; then
    echo "Convention audit FAILED."
    exit 1
fi

echo "Convention audit PASSED."
exit 0
