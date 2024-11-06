# test

## About
Test interoperability bitwen UmberDDS and RustDDS

## usage
1. create DockerNetwork
```
docker network create \
    --subnet 192.168.209.0/24 \
    --gateway 192.168.209.254 \
    docker-pcap-test
```

2. run test
```
./test.sh
```
