pub struct TypeDesc {
    name: String,
}

impl TypeDesc {
    fn new(name: String) -> Self {
        Self { name }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
