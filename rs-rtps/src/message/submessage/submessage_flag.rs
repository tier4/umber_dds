 use enumflags2;

pub enum AckNackFlag {
    Endianness = 0b01,
    Final = 0b10,
}

pub enum DataFlag {
    Endianness = 0b00001,
    InlineQos = 0b00010,
    Datqa = 0b00100,
    Key = 0b01000,
    NonStandardPayload = 0b10000,
}

pub enum DataFragFlag {
    Endianness = 0b0001,
    InlineQos = 0b0010,
    NonStandardPayload = 0b0100,
    Key = 0b1000,
}

pub enum GapFlag {
    Endianness = 0b01,
    GroupInfo = 0b10,
}

pub enum HeartbeatFlag {
    Endianness = 0b0001,
    Final = 0b0010,
    Liveliness = 0b0100,
    GroupInfo = 0b1000,
}

pub enum HeartbeatFragFlag {
    Endianness = 0b1,
}

pub enum InfoDestionationFlag {
    Endianness = 0b1,
}

pub enum InfoReplyFlag {
    Endianness = 0b01,
    Multicast = 0b10,
}

pub enum InfoSourceFlag {
    Endianness = 0b1,
}

pub enum InfoTimestampFlag {
    Endianness = 0b01,
    Invalidate = 0b10,
}

pub enum NackFragFlag {
    Endianness = 0b1,
}
