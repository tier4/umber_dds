#!/usr/bin/bash

echo "execting tests: it takes about 1 minutes"

echo "execting test1. log is save to test1.log"
./test_cases/test_case1.sh &> test1.log
res1=$?

echo "execting test2. log is save to test2.log"
./test_cases/test_case1.sh &> tes2.log
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
