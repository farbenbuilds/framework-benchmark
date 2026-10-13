#!/usr/bin/env sh
set -eu

fail=0

files() {
    git ls-files --cached --others --exclude-standard
}

check_emojis() {
    files | while IFS= read -r file; do
        [ -f "$file" ] || continue
        case $file in .agents/skills/*) continue ;; esac
        LC_ALL=C grep -P '[\x{1F300}-\x{1F9FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]' "$file" \
            >/dev/null 2>&1 && printf '%s\n' "$file"
    done
}

check_lines() {
    files | while IFS= read -r file; do
        [ -f "$file" ] || continue
        case $file in .agents/skills/*|*Cargo.lock|*/package-lock.json|skills-lock.json) continue ;; esac
        lines=$(wc -l < "$file")
        [ "$lines" -le 200 ] || printf '%s (%s lines)\n' "$file" "$lines"
    done
}

check_markers() {
    files | while IFS= read -r file; do
        [ -f "$file" ] || continue
        case $file in scripts/check_conventions.*) continue ;; esac
        case $file in *.rs|*.sh|*.toml|*.yml|*.yaml)
            grep -nE '\b(FIXME|XXX|HACK)\b' "$file" >/dev/null 2>&1 && printf '%s\n' "$file"
        esac
    done
}

check_names() {
    files | while IFS= read -r file; do
        case $file in
            docs/HOW_TO_USE.md) continue ;;
            docs/*|scripts/*)
                name=$(basename "$file")
                case $name in *[A-Z]*|*' '*) printf '%s\n' "$file" ;; esac
            ;;
        esac
    done
}

run_check() {
    label=$1
    output=$2
    echo "$label"
    [ -z "$output" ] && return
    printf '%s\n' "$output"
    fail=1
}

run_check '[1/4] emoji check' "$(check_emojis)"
run_check '[2/4] 200-line check' "$(check_lines)"
run_check '[3/4] marker check' "$(check_markers)"
run_check '[4/4] docs and scripts filename check' "$(check_names)"

if [ "$fail" -ne 0 ]; then
    echo 'Convention audit failed.' >&2
    exit 1
fi
echo 'Convention audit passed.'
