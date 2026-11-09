#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Welcome,
    AdapterSelection,
    Prepare,
    Detecting,
    Available,
    Completed,
}

impl Step {
    pub fn number(self) -> usize {
        match self {
            Self::Welcome => 1,
            Self::AdapterSelection => 2,
            Self::Prepare => 3,
            Self::Detecting => 4,
            Self::Available => 5,
            Self::Completed => 6,
        }
    }
}
