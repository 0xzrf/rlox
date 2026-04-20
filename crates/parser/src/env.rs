use std::collections::HashMap;

pub struct Env {
    values: HashMap<String, Option<String>>,
}


impl Env {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn get_var(&self, name: &str) -> Option<&Option<String>> {
        self.values.get(name)
    }

    pub fn store_var(&mut self, name: String, value: Option<String>) {
        self.values.insert(name, value);
    }
}
