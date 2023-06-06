#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubdivisionError {
    TooManySubdivisions {
        requested: u32,
        limit: u32,
    },
}

impl std::error::Error for SubdivisionError {}

impl std::fmt::Display for SubdivisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubdivisionError::TooManySubdivisions { requested, limit } => {
                write!(f, "requested {requested} mesh subdivisions but the limit is {limit}")
            }
        }
    }
}
