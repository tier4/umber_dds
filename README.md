# UmberDDS: Rust implementation of Data Distribution Service

In traditional DDS, the format of the exchanged data is defined using IDL, and DataWriter and DataReader are automatically generated from the IDL.
However, in UmberDDS, the format of the exchanged data is defined as a struct that implements `serde::{Serialize, Deserialize}`, and the DataWriter and DataReader use the generic types `DataWriter<D: Serialize>` and `DataReader<D: Deserialize>`.

## Usage
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
./target/debug/examples/shapes_demo -m p -r b
```

besteffort subscriber
```
./target/debug/examples/shapes_demo -m s
```

## Interoperability
- [x] FastDDS
- [x] RustDDS
- [ ] CycloneDDS

## Progress

- [x] RTPS Discovery Module
- [x] RTPS MessageReveiver Module
- [ ] RTPS Behavior Module
    - [x] Best-Effort StatefulWriter Behavior
    - [x] Reliable StatefulWriter Behavior
    - [ ] Best-Effort StatelessReader Behavior
    - [ ] Reliable StatelessReader Behavior
- [ ] RTPS Writer Liveliness Protocol
- [ ] Logging
- [ ] Topics kinds: with_key and no_key

### Supporting QoS

Unsuporting QoS can be set, but it dosn't effect behavior of UmberDDS.

- [ ] Reliability
    - [x] kind (Reliability, BestEffort)
    - [ ] max_bloking_time
- [ ] DurabilityService
- [ ] Durability
- [ ] Presentaion
- [ ] Deadline
- [ ] LatencyBudget
- [ ] Ownership
- [ ] OwnershipStrength
- [ ] Liveliness
- [ ] TimeBasedFilter
- [ ] DestinationOrder
- [ ] History
- [ ] ResourceLimits
- [ ] Lifespan
- [ ] Partition
- [ ] UserData
- [ ] TopicData
- [ ] GrupData
- [ ] WriterDataLifecycle
- [ ] ReaderDataLifecycle
- [ ] TransportPrioriry
- [ ] EntityFactory


