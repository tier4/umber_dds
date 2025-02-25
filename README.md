# UmberDDS: An experimental Rust implementation of Data Distribution Service

In traditional DDS, the format of the exchanged data is defined using IDL, and DataWriter and DataReader are automatically generated from the IDL.
However, in UmberDDS, the format of the exchanged data is defined as a struct that implements `serde::{Serialize, Deserialize}`, and the DataWriter and DataReader use the generic types `DataWriter<D: Serialize>` and `DataReader<D: Deserialize>`.

## Usage
If you want to use refarence/dds-analysis-docker to perform communication tests with other implementations or analyze this implementation in a Docker environment, or if you want to test interoperability with RustDDS before contributing using test, several dependencies registered as Git submodules are required.

When cloning this repository, please use the --recursive option to clone the dependencies along with it.
```
git clone --recursive https://github.com/tier4/umber_dds.git
```

UmberDDS has not yet implemented proper logging, so all debug information is output directly to `stderr`. Therefore, if you do not need debug information, it is recommended to redirect `stderr` to `/dev/null`.

## How to define exchanged data
I'll explain using the following IDL as an example.
```
// Shape.idl
struct ShapeType
{
  @key string color;
  long x;
  long y;
  long shapesize;
};
```

The structure representing the exchanged data must implement three traits: `serde::{Serialize, Deserialize}`, and `umberdds::DdsData`.
`serde::{Serialize, Deserialize}` is necessary for serializing and deserializing the data into the RTPS message format.
`umberdds::DdsData` is required to specify the DataType name and the key.
If some keys exists, annotate it with `#[key]`.
If the structure name does not match the DataType name, specify the DataType name using `#[dds_data(type_name = "{name}")]`.
```
use umber_dds::{DdsData, kye::KeyHash};
use md5::compute;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, DdsData)]
#[dds_data(type_name = "ShapeType")]
struct Shape {
    #[key]
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}
```

### examples/shapes_deme
```
# build examples
cargo build --examples
```

You can choose to start either a Publisher or a Subscriber using `-m`.
To start the Publisher, use `-m p` or `-m P`. To start the Subscriber, use `-m s` or `-m S`.

You can specify the reliability of the entity with `-r`.
To specify Reliable, use `-r r` or `-r R`. To specify BestEffort, use `-r b` or `-r B`.
The default is BestEffort.

reliable publisher
```
./target/debug/examples/shapes_demo -m p -r b 2> /dev/null
```

besteffort subscriber
```
./target/debug/examples/shapes_demo -m s 2> /dev/null
```

## Interoperability
- [x] Fast DDS (%1)
- [x] RustDDS (%1)
- [x] Cyclone DDS (%1)

(%1) These implementations use IPC (Inter-Process Communication) instead of UDP for communication when Participants are on the same host. Since UmberDDS does not implement IPC, it cannot communicate with Participants of these implementations on the same host. If you need to communicate with these implementations on the same host, disable IPC or run UmberDDS in an isolated network namespace using tools like Docker.


## Progress

- [x] RTPS Discovery Module
- [x] RTPS MessageReveiver Module
- [ ] RTPS Behavior Module
    - [x] Best-Effort StatefulWriter Behavior
    - [x] Reliable StatefulWriter Behavior
    - [ ] Best-Effort StatelessReader Behavior
    - [ ] Reliable StatelessReader Behavior
- [x] RTPS Writer Liveliness Protocol
- [ ] Logging
- [x] Topics kinds: with_key and no_key
- [ ] Instance
- [ ] IPC (Inter-Process Communication)

### Supporting QoS

Unsuporting QoS can be set, but it dosn't effect behavior of UmberDDS.

#### ROS 2 suported QoS
- [ ] Reliability
    - [x] kind (Reliability, BestEffort)
    - [ ] max_bloking_time
- [ ] Durability
- [ ] Deadline
- [ ] History
- [ ] Lifespan
- [x] Liveliness (ManualByTopic is not suported)

#### ROS 2 unsuported QoS
- [ ] DurabilityService
- [ ] Presentaion
- [ ] LatencyBudget
- [ ] Ownership
- [ ] OwnershipStrength
- [ ] TimeBasedFilter
- [ ] DestinationOrder
- [ ] ResourceLimits
- [ ] Partition
- [ ] UserData
- [ ] TopicData
- [ ] GrupData
- [ ] WriterDataLifecycle
- [ ] ReaderDataLifecycle
- [ ] TransportPrioriry
- [ ] EntityFactory

