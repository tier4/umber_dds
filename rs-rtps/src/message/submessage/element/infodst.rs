use crate::structure::guid::*;
use speedy::Readable;

#[derive(Readable)]
pub struct InfoDestination {
    pub guidPrefix: GuidPrefix,
}
