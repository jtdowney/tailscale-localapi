#!/bin/sh
set -e

socket=/var/run/tailscale/tailscaled.sock

tailscaled --tun=userspace-networking --statedir=/var/lib/tailscale --socket="$socket" &
pid=$!
trap 'kill "$pid"' INT TERM

until tailscale --socket="$socket" up \
  --login-server=http://headscale:8080 \
  --auth-key="$(cat /auth/authkey)" \
  --hostname="$TS_HOSTNAME" \
  --accept-dns=false; do
  sleep 1
done

wait "$pid"
