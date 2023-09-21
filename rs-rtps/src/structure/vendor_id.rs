use speedy::{Readable, Writable};

#[derive(Readable, Writable, Debug)]
pub struct VendorId {
    pub vendor_id: [u8; 2],
}

impl VendorId {
    // not assigned by OMG DDS SIG yet
    // In 2023/03/09 [0x01, 0x01] to [0x01, 0x13] is reserved
    // https://www.dds-foundation.org/dds-rtps-vendor-and-product-ids/
    // TODO: contact OMG to get vender ID
    pub const TIER4: Self = Self {
        vendor_id: [0x01, 0x20],
    };

    pub const THIS_IMPLEMENTATION: Self = Self::TIER4;

    pub const VENDORID_UNKNOW: Self = Self {
        vendor_id: [0x00; 2],
    };
}
