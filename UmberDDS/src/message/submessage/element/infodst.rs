use crate::structure::guid::*;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct InfoDestination {
    pub guid_prefix: GuidPrefix,
}
