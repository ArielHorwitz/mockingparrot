#! /bin/bash
set -e

printerr() { printf '\x1b[1;31m%s\x1b[m\n' "$@" ; }
printok() { printf '\x1b[1;32m%s\x1b[m\n' "$@" ; }
try_command() {
    set -e
    local name=$1
    shift
    if "$@" ; then
        printok " ✔ $name"
        return 0
    else
        printerr " ✘ $name"
        return 1
    fi
}

EXPECTED_CARGO_VERSION="1.87.0"
actual_cargo_version=$(cargo --version | awk '{print $2}')
if [[ "$actual_cargo_version" != "$EXPECTED_CARGO_VERSION" ]]; then
    printerr "Unexpected cargo version"
    echo "Expected: $EXPECTED_CARGO_VERSION, found: $actual_cargo_version"
    exit 1
fi

clippy_cmd=(
    cargo clippy
    --workspace
    --all-targets
    --all-features
    --color always
    --
    --deny clippy::pedantic
    --allow clippy::missing_errors_doc
)
try_command "Clippy" "${clippy_cmd[@]}"
format_cmd=(cargo fmt --check)
try_command "Format" "${format_cmd[@]}"
tests_cmd=(
    cargo test
    --workspace
    --all-targets
    --all-features
    --quiet
    --color always
    --
    --color always
)
try_command "Tests" "${tests_cmd[@]}"
