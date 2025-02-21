#!/usr/bin/bash

sudo docker compose up -d

declare -a pids=()
declare -i results=0

sudo docker exec -i otherdds /home/shapes_demo_cyclonedds/ShapeDemoSubscriber &
pids+=($!)

sudo docker exec -i umberdds /home/UmberDDS/target/debug/examples/shapes_demo_for_autotest -m p &
pids+=($!)

for pid in "${pids[@]}"; do
    wait "$pid"
    results+=$?
done

exit $results
