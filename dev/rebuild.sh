#!/bin/sh
# build the shell and restart the copy running inside the nested session
cd "$(dirname "$0")" || exit 1
export PATH="$HOME/.cargo/bin:$PATH"
(cd .. && amane compile) || exit 1

. ./.envload.sh

pkill -xf "$PWD/.amane-shell-nested"
sleep 0.5
cp "$HOME/.cache/amane/amane-shell" "$PWD/.amane-shell-nested"
setsid "$PWD/.amane-shell-nested" > /tmp/amane-nested.log 2>&1 < /dev/null &
