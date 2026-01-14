#!/usr/bin/bash

sudo docker compose up -d

declare -a pids=()
declare -i results=0
declare -A proc_labels

sudo docker exec -i umber_dds /home/umber_dds/target/debug/examples/shapes_demo_for_autotest -m s &
pid1=$!
pids+=($pid1)
proc_labels[$pid1]=" shapes_demo_for_autotest (umber_dds, Sub)"

sudo docker exec -i otherdds /home/shapes_demo_rustdds/target/debug/shapes_demo_rustdds -t Square -P &
pid2=$!
pids+=($pid2)
proc_labels[$pid2]="shapes_demo_rustdds (rustdds, Pub)"

for pid in "${pids[@]}"; do
    wait "$pid"
    exit_code=$?

    label=${proc_labels[$pid]}
    echo "[Exit Code: $exit_code] - $label"

    results+=$exit_code
done

exit $results
