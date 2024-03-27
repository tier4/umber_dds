#[derive(Clone)]
pub struct TypeDesc {
    name: String,
}

impl TypeDesc {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
