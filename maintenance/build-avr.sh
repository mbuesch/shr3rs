#!/bin/sh
set -e
basedir="$(realpath -e "$0" | xargs dirname)"

cd "$basedir/.."
cargo build --target "$basedir/avr-atmega328p.json" -Z build-std=core --release
