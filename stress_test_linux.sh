#!/bin/bash
# Stress test (Linux)
# Requires siege: https://www.joedog.org/siege-home

app="metricore"
store="./events.txt"

cargo build --release
result=$?
if [ $result -ne 0 ]; then
    exit $result
fi

rm -f $store
./target/release/$app &

siege -b --concurrent 10 --content-type="application/json" 'http://localhost:3000 PUT {"event":"TEST"}' -t 60s

killall $app
wc -l $store

echo "Done"
