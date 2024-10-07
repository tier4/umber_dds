#!/usr/bin/bash

echo "preparing tests: starting docker containers"
sudo docker compose up -d
echo "tests start: it takes about 1 minutes"

echo "execting test1. log is save to test1.log"
./test_cases/test_case1.sh &> test1.log
res1=$?

echo "execting test2. log is save to test2.log"
./test_cases/test_case1.sh &> test2.log
res2=$?

function show_resut() {
    if [ "$1" -eq 0 ];then
        echo "succeeded"
    else
        echo "failed"
    fi
}

echo "test_case1: RustDDS to UmberDDS"
show_resut $res1

echo "test_case2: UmberDDS to RustDDS"
show_resut $res2
