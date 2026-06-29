# Umber DDS: An experimental Rust implementation of Data Distribution Service

In traditional DDS, the format of the exchanged data is defined using IDL, and DataWriter and DataReader are automatically generated from the IDL.
However, in Umber DDS, the format of the exchanged data is defined as a struct that implements `serde::{Serialize, Deserialize}`, and the DataWriter and DataReader use the generic types `DataWriter<D: Serialize>` and `DataReader<D: Deserialize>`.

## Usage
If you want to use refarence/dds-analysis-docker to perform communication tests with other implementations or analyze this implementation in a Docker environment, or if you want to test interoperability with RustDDS before contributing using test, several dependencies registered as Git submodules are required.

When cloning this repository, please use the --recursive option to clone the dependencies along with it.
```
git clone --recursive https://github.com/tier4/umber_dds.git
```

Umber DDS implement logging using [log crate](https://docs.rs/log/latest/log/).
You can log behavior of the Umber DDS using logger implementation compatible with the facade.

Each log level is utilized as follows:

User-Facing Logs: Recommended for most users to monitor system health and behavior.

+ error: Indicates critical issues that require immediate attention or code changes.

+ warn: Highlights potential problems or unexpected behaviors that may require investigation.

+ info: Tracks the high-level operational flow of the user's program. This primarily focuses on the DDS layer to confirm everything is running as intended.

Developer-Facing Logs: Highly detailed logs intended for library contributors and debugging deep-seated issues.

+ debug: Provides insights into the internal state and logic transitions of the library.

+ trace: Offers the most granular details, specifically tracking RTPS layer events such as individual message reception, timer triggers, and low-level network activity.



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

To represent this exchanged data in Umber DDS, your Rust structure must use the `DdsData`, `DdsSerialize`, and `DdsDeserialize` derive macros provided by `umber_dds`.

Crucially, to send and receive messages in DDS, the data must be serialized and deserialized strictly following the OMG CDR (Common Data Representation) format.
Simply deriving the standard `speedy::{Writable, Readable}` on your structure will not produce the correct CDR format due to specific alignment and padding rules.
Instead, you must use the `DdsSerialize` and `DdsDeserialize` macros. These automatically generate the proper `Writable` and `Readable` implementations to guarantee CDR-compliant memory alignment.

### Configuration & Data Types
+ Type Name: The `umber_dds::DdsData` trait specifies the DataType name and keys. If the Rust structure name does not match the target DataType name, specify it using `#[dds_data(type_name = "{name}")]`.
+ Keys: Annotate key fields with `#[key]`. The macro uses these fields to generate a unique 16-byte `KeyHash` (Note: Your project must depend on the `md5` crate in `Cargo.toml` for this functionality).
+ Characters: Be aware that Rust's `char` type is serialized and deserialized as a 1-byte C-style character (equivalent to `u8`), not as a 4-byte Unicode scalar value.

### IDL to Rust Type Mapping

When defining your data structures in Rust, use the following mapping to ensure your types correspond correctly to standard OMG IDL types. Umber DDS's `DdsSerialize` and `DdsDeserialize` macros automatically handle the strict memory alignment and padding required by the CDR (Common Data Representation) format for these types.

| IDL Type | Rust Type (Umber DDS) | Notes |
| :--- | :--- | :--- |
| `octet` | `u8` | 1-byte unsigned integer. |
| `char` | `char` or `u8` | Serialized and deserialized as a 1-byte C-style character. *Note: Rust's `char` is normally a 4-byte Unicode scalar, but the macro strictly treats it as 1-byte for DDS compatibility.* |
| `boolean` | `bool` | 1-byte boolean value. |
| `short` | `i16` | 2-byte signed integer. |
| `unsigned short` | `u16` | 2-byte unsigned integer. |
| `long` | `i32` | 4-byte signed integer. |
| `unsigned long` | `u32` | 4-byte unsigned integer. |
| `long long` | `i64` | 8-byte signed integer. |
| `unsigned long long` | `u64` | 8-byte unsigned integer. |
| `float` | `f32` | 4-byte floating point. |
| `double` | `f64` | 8-byte floating point. |
| `string` | `String` | Automatically handles length prefixes and null-termination. |
| `sequence<T>` | `Vec<T>` | Dynamically sized array of type `T`. |
| `T[N]` | `[T; N]` | Fixed-size array of type `T` with length `N`. |
| `map<K, V>` | `HashMap<K, V>` / `BTreeMap<K, V>` | Key-value mapping. |
| `struct` | `struct` | Nested structs must also derive `speedy::Readable` / `speedy::Writable` (or `DdsSerialize`/`DdsDeserialize`). |

### Example Code
Ensure that you bring `speedy::Writable` and `umber_dds::KeyHash` into scope, as the generated macro code relies on them internally.
```
use speedy::Writable;
use umber_dds::{DdsData, DdsSerialize, DdsDeserialize, KeyHash};

#[derive(DdsData, DdsSerialize, DdsDeserialize)]
#[dds_data(type_name = "ShapeType")]
struct Shape {
    #[key]
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}
```

## examples
```
# build examples
cargo build --examples
```

### shapes_demo

This can be used to demonstrate the capabilities of Umber DDS or as a proof of interoperability with other DDS/RTPS-compliant implementations.

ShapesDemo of Fast DDS: https://github.com/eProsima/ShapesDemo
ShapesDemo of RustDDS: https://github.com/Atostek/RustDDS/tree/master/examples/shapes_demo

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

At the beginning of this program, logging for Umber DDS behavior is initialized using log4rs. If the `shapes_logging.yml` file is found, its configuration is used; otherwise, logs whose level is Warn or higher are output to the console.

#### shapes_demo_for_autotest

This is used for test/test.sh.

## Interoperability
- [x] Fast DDS
- [x] RustDDS
- [x] Cyclone DDS

## Progress

- [x] RTPS Discovery Module
- [x] RTPS MessageReveiver Module
- [ ] RTPS Behavior Module
    - [x] Best-Effort StatefulWriter Behavior
    - [x] Reliable StatefulWriter Behavior
    - [ ] Best-Effort StatelessReader Behavior
    - [ ] Reliable StatelessReader Behavior
- [x] RTPS Writer Liveliness Protocol
- [x] Logging
- [x] Topics kinds: with_key and no_key
- [ ] Instance
- [ ] InlineQoS

### Supporting QoS

Unsuporting QoS can be set, but it dosn't effect behavior of Umber DDS.

#### ROS 2 suported QoS
- [ ] Reliability
    - [x] kind (Reliability, BestEffort)
    - [ ] max_bloking_time
- [X] Durability (Volatile, TransientLocal only)
- [x] Deadline
- [X] History
- [X] Lifespan
- [x] Liveliness (ManualByTopic is not suported)

#### ROS 2 unsuported QoS
- [ ] DurabilityService
- [ ] Presentaion
- [ ] LatencyBudget
- [ ] Ownership
- [ ] OwnershipStrength
- [ ] TimeBasedFilter
- [ ] DestinationOrder
- [X] ResourceLimits
- [ ] Partition
- [ ] UserData
- [ ] TopicData
- [ ] GrupData
- [ ] WriterDataLifecycle
- [ ] ReaderDataLifecycle
- [ ] TransportPrioriry
- [ ] EntityFactory

