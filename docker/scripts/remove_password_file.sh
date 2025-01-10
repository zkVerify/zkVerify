#!/usr/bin/env bash

echo "Waiting for node to launch"
until pid=$(pidof $1)
do
    sleep 1
done

echo "Waiting for node to complete startup"
cat /proc/${pid}/fd/2 | grep -qe "🏆 Imported"

if [[ $? == 0 ]]; then
    echo "Node started and synced; wiping password"
    #Remove the file. Alternatively, we could 'echo "=== removed ===" > $2'
    rm -f $2
fi
