#!/bin/sh
set -e

headscale users create test

umask 077
headscale preauthkeys create --user 1 --reusable -e 30m > /auth/authkey

trap 'exit 0' INT TERM
/busybox/sleep 86400 &
wait
