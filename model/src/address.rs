use std::fmt;

#[derive(Debug)]
pub struct Address {
    pub places: Vec<String>,
    pub postcode: Option<String>,
    pub state: State,
}

#[derive(Debug)]
pub enum State {
    NSW,
    VIC,
    QLD,
    SA,
    WA,
    TAS,
    NT,
    ACT,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NSW => write!(f, "NSW"),
            Self::VIC => write!(f, "VIC"),
            Self::QLD => write!(f, "QLD"),
            Self::SA => write!(f, "SA"),
            Self::WA => write!(f, "WA"),
            Self::TAS => write!(f, "TAS"),
            Self::NT => write!(f, "NT"),
            Self::ACT => write!(f, "ACT"),
        }
    }
}
