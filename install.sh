#!/bin/sh

basedir="$(realpath "$0" | xargs dirname)"

die() {
    echo "$*" >&2
    exit 1
}

if [ "$(id -u)" = "0" ]; then
    die "Must NOT be root to install."
fi

exec cargo install --path "$basedir"

# vim: ts=4 sw=4 expandtab
