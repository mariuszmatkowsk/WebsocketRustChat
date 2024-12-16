#[derive(Clone, Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    pub fn new(name: String, value: String) -> Self {
        Self {name, value}
    }

    pub fn default() -> Self {
        Self {name: String::new(), value: String::new()}
    }
}
