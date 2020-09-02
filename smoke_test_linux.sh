#!/bin/bash
# Smoke test (Linux)
# Requires httpie: https://httpie.org

app="metricore"
store="./events.txt"

cargo build
result=$?
if [ $result -ne 0 ]; then
    exit $result
fi

./target/debug/$app &

http PUT localhost:3000 event=A
sleep 1s
http PUT localhost:3000 event=B
sleep 1s
http PUT localhost:3000 event=C
sleep 1s
http PUT localhost:3000 event=D

echo "Waiting 5 seconds..."
sleep 5s

killall $app
cat $store

echo "Done"
