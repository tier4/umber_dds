# umber_dds/refarence
Resources referenced during implementation

## RustDDS
DDS implemented Rust.

## ShapesDemo
ShapesDemo implemented FastDDS

## dds-analysis-docker
docker environmet for analysis rtps

## SampleAnalysis.md
Analysis note. (Japanese)

## How to analysis DDS
0. Install prerequisites

Install Docker & rust toolchain On Host

1. Compile shapes_demo implemented RustDDS
```
# On RustDDS
cargo build --example=shapes_demo
```

Proceed with the work in dds-analysis-docker
2. Launch Docker contairs
```
./build.sh # build docker images
sudo docker compose up -d
```

3. Enter publisher contair & setup ShapesDemo
```
sudo docerk exec -it dds_pub bash
. ShapesDemo/install/setup.bash
```

4. Open other terminal & enter subscriber contair
```
sudo docerk exec -it dds_sub bash
```

5. Launch FastDDS ShapesDemo on publisher contair
```
ShapesDemo
```

6. Launch RustDDS ShapesDemo on subscriber contair
```
./target/debug/examples/shapes_demo -S -t Square
```
Or launch on debuger
```
gdb ./target/debug/examples/shapes_demo
(gdb) r -S -t Square
```

