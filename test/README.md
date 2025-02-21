# test

## About
Test interoperability between UmberDDS and RustDDS, between UmberDDS and UmberDDS and between UmberDDS and Cyclone DDS

The test code for UmberDDS is located in `example/shapes_demo_for_autotest.rs`, while the test code for RustDDS is in `./otherdds/shapes_demo_rustdds`, while the test code for Cyclone DDS is in `./otherdds/shapes_demo_cyclonedds`.

The test involves running a Publisher and a Subscriber simultaneously. The Publisher publishes the Square Topic, while the Subscriber subscribes to the Square Topic, and both processes terminate after 30 seconds. If the Subscriber receives 5 messages before it terminates, the test is considered successful; otherwise, it is considered a failure.

## usage
1. create DockerNetwork
```
docker network create \
    --subnet 192.168.209.0/24 \
    --gateway 192.168.209.254 \
    docker-pcap-test
```

2. build docker container
```
./build.sh
```

3. run test
```
./test.sh
```
