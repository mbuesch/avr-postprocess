#!/bin/sh
basedir="$(realpath "$0" | xargs dirname)"
exec cargo install --path "$(basedir)"
# vim: ts=4 sw=4 expandtab
