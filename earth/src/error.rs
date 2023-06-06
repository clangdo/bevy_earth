pub enum ArgumentParseError {
    ExpectedAt,
    ExpectedLayout,
    LayoutParseError,
    GridVecParseError,
}

impl std::fmt::Display for ArgumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let message = match self {
            ArgumentParseError::ExpectedAt => "expected an \"at\" before location of new tile",
            ArgumentParseError::ExpectedLayout => "expected \"layout\" after biome name",
            ArgumentParseError::LayoutParseError => "malformed layout argument",
            ArgumentParseError::GridVecParseError => "malformed grid vector argument",
        };

        write!(f, "{}", message)
    }
}
