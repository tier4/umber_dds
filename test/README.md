# test

## About
Test interoperability between Umber DDS and RustDDS, between Umber DDS and Umber DDS and between Umber DDS and Cyclone DDS

The test code for Umber DDS is located in `example/shapes_demo_for_autotest.rs`, while the test code for RustDDS is in `./otherdds/shapes_demo_rustdds`, while the test code for Cyclone DDS is in `./otherdds/shapes_demo_cyclonedds`.

The test involves running a Publisher and a Subscriber simultaneously. The Publisher publishes the Square Topic, while the Subscriber subscribes to the Square Topic, and both processes terminate after 30 seconds. If the Subscriber receives 5 messages before it terminates, the test is considered successful; otherwise, it is considered a failure.

## usage
1. create DockerNetwork
```
docker network create \
    --subnet 192.168.209.0/24 \
    --gateway 192.168.209.254 \
    docker-pcap-test
```

and write name of created NIC to test_nic
```
echo "TEST_NIC={name of NIC}" > test_nic
```

2. build docker container
```
./build.sh
```

3. run test
```
./test.sh
```

If you only want to run a specific test_case, pass a number to test.sh in which the bit corresponding to that test_case’s index is set.
For example, to run test_case2 and test_case3, you would do:
```
# run test_case2 and test_case3
# 0b110 = 6
./test.sh 6
```
