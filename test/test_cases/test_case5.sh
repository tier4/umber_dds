#!/usr/bin/bash

sudo docker compose up -d

declare -a pids=()
declare -i results=0

sudo docker exec -i otherdds /home/shapes_demo_cyclonedds/ShapesDemoSubscriber &
pids+=($!)

sudo docker exec -i umber_dds /home/umber_dds/target/debug/examples/shapes_demo_for_autotest -m p &
pids+=($!)

for pid in "${pids[@]}"; do
    wait "$pid"
    results+=$?
done

exit $results
