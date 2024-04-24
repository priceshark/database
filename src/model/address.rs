use std::fmt;

#[derive(Debug)]
pub struct Address {
    pub places: Vec<String>,
    pub postcode: Option<String>,
    pub state: State,
}
