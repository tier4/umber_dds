use crate::structure::GuidPrefix;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct InfoDestination {
    pub guid_prefix: GuidPrefix,
}
