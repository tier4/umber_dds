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

source test_nic

test_count=0
success_count=0

if [ $# -gt 0 ]; then
    for i in {0..4}; do
        ibit=$((1<<$i))
        if [ $(($1&$ibit)) -gt 0 ]; then
            test_count=$((test_count+=1))
        fi
    done
    if [ $test_count -eq 0 ]; then
        echo "[test.sh] no test case to run"
        exit
    fi
    echo "[test.sh] tests start: it takes about 0.5 * $test_count minutes"
else
    test_count=5
    echo "[test.sh] tests start: it takes about 2.5 minutes"
fi

if [ $# -eq 0 ] || [ $(($1&1)) -gt 0 ]; then
    echo "[test.sh] execting test1. log is save to test1.log. packet capture is save to capture1.pcap."
    sudo tcpdump -i $TEST_NIC -w capture1.pcap &
    TCPDUMP_PID=$!
    sleep 2
    ./test_cases/test_case1.sh &> test1.log
    res1=$?
    sudo pkill -SIGINT -P "$TCPDUMP_PID"
    wait "$TCPDUMP_PID" 2>/dev/null
else
    res1=0
fi

if [ $# -eq 0 ] || [ $(($1&2)) -gt 0 ]; then
    echo "[test.sh] execting test2. log is save to test2.log. packet capture is save to capture2.pcap."
    sudo tcpdump -i $TEST_NIC -w capture2.pcap &
    TCPDUMP_PID=$!
    sleep 2
    ./test_cases/test_case2.sh &> test2.log
    res2=$?
    sudo pkill -SIGINT -P "$TCPDUMP_PID"
    wait "$TCPDUMP_PID" 2>/dev/null
else
    res2=0
fi

if [ $# -eq 0 ] || [ $(($1&4)) -gt 0 ]; then
    echo "[test.sh] execting test3. log is save to test3.log. packet capture is save to capture3.pcap."
    sudo tcpdump -i $TEST_NIC -w capture3.pcap &
    TCPDUMP_PID=$!
    sleep 2
    ./test_cases/test_case3.sh &> test3.log
    res3=$?
    sudo pkill -SIGINT -P "$TCPDUMP_PID"
    wait "$TCPDUMP_PID" 2>/dev/null
else
    res3=0
fi

if [ $# -eq 0 ] || [ $(($1&8)) -gt 0 ]; then
    echo "[test.sh] execting test4. log is save to test4.log. packet capture is save to capture4.pcap."
    sudo tcpdump -i "$TEST_NIC" -w capture4.pcap &
    TCPDUMP_PID=$!
    sleep 2
    ./test_cases/test_case4.sh &> test4.log
    res4=$?
    sudo pkill -SIGINT -P "$TCPDUMP_PID"
    wait "$TCPDUMP_PID" 2>/dev/null
else
    res4=0
fi

if [ $# -eq 0 ] || [ $(($1&16)) -gt 0 ]; then
    echo "[test.sh] execting test5. log is save to test5.log. packet capture is save to capture5.pcap."
    sudo tcpdump -i $TEST_NIC -w capture5.pcap &
    TCPDUMP_PID=$!
    sleep 2
    ./test_cases/test_case5.sh &> test5.log
    res5=$?
    sudo pkill -SIGINT -P "$TCPDUMP_PID"
    wait "$TCPDUMP_PID" 2>/dev/null
else
    res5=0
fi

function show_resut() {
    if [ "$1" -eq 0 ];then
        echo -e "[test.sh] \e[0;32msucceeded\e[0m"
        success_count=$((success_count+=1))
    else
        echo -e "[test.sh] \e[0;31mfailed\e[0m"
    fi
}

if [ $# -eq 0 ] || [ $(($1&1)) -gt 0 ]; then
    echo "[test.sh] test_case1: RustDDS to UmberDDS"
    show_resut $res1
fi

if [ $# -eq 0 ] || [ $(($1&2)) -gt 0 ]; then
    echo "[test.sh] test_case2: UmberDDS to RustDDS"
    show_resut $res2
fi

if [ $# -eq 0 ] || [ $(($1&4)) -gt 0 ]; then
    echo "[test.sh] test_case3: UmberDDS to UmberDDS"
    show_resut $res3
fi

if [ $# -eq 0 ] || [ $(($1&8)) -gt 0 ]; then
    echo "[test.sh] test_case4: Cyclone DDS to UmberDDS"
    show_resut $res4
fi

if [ $# -eq 0 ] || [ $(($1&16)) -gt 0 ]; then
    echo "[test.sh] test_case5: UmberDDS to Cyclone DDS"
    show_resut $res5
fi


echo "[test.sh] $success_count/$test_count succeeded"
if [ "$success_count" -eq "$test_count" ];then
    echo -e "[test.sh] \e[0;32mall test succeeded\e[0m"
    exit 0
else
    echo -e "[test.sh] \e[0;31m$(($test_count-$success_count))/$test_count failed\e[0m"
    exit -1
fi
