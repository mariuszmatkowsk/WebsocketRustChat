#[derive(Clone, Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    pub fn new() -> Self {
        Self {name: String::new(), value: String::new()}
    }
}
