#!/usr/bin/bash

echo "[test.sh] preparing tests: building umberdds/example/shapes_demo_for_autotest"
cargo build --example=shapes_demo_for_autotest
if [ "$?" -ne 0 ];then
    echo "[test.sh] shapes_demo_for_autotest build failed!"
    exit
fi

echo "[test.sh] preparing tests: building otherdds/shapes_demo_rustdds"
cd otherdds/shapes_demo_rustdds
cargo build
if [ "$?" -ne 0 ];then
    echo "[test.sh] shapes_demo_rustdds build failed!"
    exit
fi
cd ../..

echo "[test.sh] preparing tests: starting docker containers"
sudo docker compose up -d
if [ "$?" -ne 0 ];then
    echo "[test.sh] start containers failed!"
    exit
fi
echo "[test.sh] tests start: it takes about 1.5 minutes"

echo "[test.sh] execting test1. log is save to test1.log"
./test_cases/test_case1.sh &> test1.log
res1=$?

echo "[test.sh] execting test2. log is save to test2.log"
./test_cases/test_case2.sh &> test2.log
res2=$?

echo "[test.sh] execting test3. log is save to test3.log"
./test_cases/test_case3.sh &> test3.log
res3=$?

function show_resut() {
    if [ "$1" -eq 0 ];then
        echo "[test.sh] succeeded"
    else
        echo "[test.sh] failed"
    fi
}

echo "[test.sh] test_case1: RustDDS to UmberDDS"
show_resut $res1

echo "[test.sh] test_case2: UmberDDS to RustDDS"
show_resut $res2

echo "[test.sh] test_case3: UmberDDS to UmberDDS"
show_resut $res3
