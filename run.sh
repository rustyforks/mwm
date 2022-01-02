#!/bin/sh

_DISPLAY=:3
_RESOLUTION=800x600
_STACKTRACE=1

Xephyr +extension RANDR -screen $_RESOLUTION $_DISPLAY &
xephyr_pid=$!

DISPLAY=$_DISPLAY \
RUST_BACKTRACE=$_STACKTRACE \
RUST_LOG="${RUST_LOG:-debug}" \
    cargo run $@
echo $?

kill $xephyr_pid
