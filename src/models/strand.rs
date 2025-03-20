#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Strand {
    Forward,
    Reverse,
}

impl Strand {
    pub fn from_str(s: String) -> Result<Self, ()> {
        match s.as_str() {
            "+" => Ok(Strand::Forward),
            "-" => Ok(Strand::Reverse),
            _ => Err(()),
        }
    }
}
