pub mod acknack;
pub mod data;
pub mod datafrag;
pub mod gap;
pub mod heartbeat;
pub mod heartbeatfrag;
pub mod infodst;
pub mod inforeply;
pub mod infosrc;
pub mod infots;
pub mod nackfrag;

use bytes::Bytes;
use speedy::Readable;

// spec 9.4.2 Mapping of the PIM SubmessageElements

pub type Count = i32;

pub type SequenceNumber = i64;
pub type FragmentNumber = u32;

pub type SequenceNumberSet = NumberSet<SequenceNumber>;
pub type FragmentNumberSet = NumberSet<FragmentNumber>;

#[derive(Readable)]
struct NumberSet<T> {
    bitmap_base: T,
    num_bits: u32,
    bitmap: Vec<u32>,
}

pub type ParameterId = i16;
pub struct Parameter {
    parameterId: ParameterId,
    length: i16,
    value: Vec<u8>,
}
pub type ParameterList = Vec<Parameter>;

#[derive(Readable)]
pub struct Timestamp {
    seconds: u32,
    fraction: u32,
}

impl Timestamp {
    pub const TIME_INVALID: Self = Self {
        seconds: 0x00,
        fraction: 0x00,
    };
}

// spec versin 2.3 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data
#[derive(Readable)]
pub struct Locator {
    kind: i64,
    port: u64,
    address: [u8; 16],
}

impl Locator {
    pub const KIND_INVALID: i64 = -1;
    pub const KIND_RESERVED: i64 = 0;
    pub const KIND_UDPv4: i64 = 1;
    pub const KIND_UDPv6: i64 = 2;
    pub const PORT_INVALID: u64 = 0;
    pub const ADDRESS_INVALID: [u8; 16] = [0; 16];
    pub const INVALID: Self = Self {
        kind: Self::KIND_INVALID,
        port: Self::PORT_INVALID,
        address: Self::ADDRESS_INVALID,
    };
}
pub type LocatorList = Vec<Locator>;

pub struct RepresentationIdentifier {
    bytes: [u8; 2],
}
pub struct SerializedPayload {
    pub representation_identifier: RepresentationIdentifier,
    pub representation_options: [u8; 2], // Not used. Send as zero, ignore on receive.
    pub value: Bytes,
}

pub type GroupDigest = [u8; 4];
