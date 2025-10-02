# dds-analysis-docker

## About
docker container for analysis behavior of some DDS implementations

## usage

1. Create DockerNetwork
```
docker network create \
--subnet 192.168.208.0/24 \
--gateway 192.168.208.254 \
docker-pcap
```

3. build images
```
./build.sh
```

3. Start container
```
sudo docker compose up -d
```

3. Attach Container
```
sudo docker exec -it dds_pub bash
sudo docker exec -it dds_sub bash
```

4. Run DDS programs
Run DDSHelloWorld
```
cd DDSHelloWorld/build
# On dds_pub
./DDSHelloWorldPublisher
# On dds_sub
./DDSHelloWorldSubscriber
```
or Run Umber DDS
```
# On host
cd /path/to/umber_dds
cargo build --examples
# On dds_pub
./umber_dds/target/debug/examples/shapes_demo -m p
# On dds_sub
./umber_dds/target/debug/examples/shapes_demo -m s
```
or Run FastDDS ShapesDemo
```
# On dds_pub
. ShapesDemo/install/setup.bash
ShapesDemo
```
or Run RustDDS ShapesDemo
```
# On host
cd /path/to/RustDDS
cargo build --examples
# On dds_sub
./RustDDS/target/debug/examples/shapes_demo -t Square -P
# or
./RustDDS/target/debug/examples/shapes_demo -t Square -S
```
