use std::str::FromStr;

#[derive(Hash, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Update,
    Delete,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "UPDATE" => Ok(Self::Update),
            "DELETE" => Ok(Self::Delete),
            _ => Err(()),
        }
    }
}
