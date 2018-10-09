#!/bin/bash

set -e

# start webserver
trap 'echo "killing webserver" && kill %1' INT EXIT
cd dist && python3 ../server.py &
sleep 0.5s

# watch
REBUILD=$(cat <<-EOM
echo ""
echo ""
echo "... rebuilding ----------------"
./build.sh
xdotool search --name "Mozilla Firefox" windowactivate \
    %1 key F5 \
    windowactivate $(xdotool search --name "\(cantucci\) - Sublime Text")
EOM
)

watchexec \
    -w "src" \
    "$REBUILD"
