use std::fmt;

#[derive(Debug)]
pub struct PhotoInsightError {
    pub message: String,
}

impl PhotoInsightError {
    pub fn new<E: std::error::Error>(err: E) -> Self {
        PhotoInsightError {
            message: err.to_string(),
        }
    }

    pub fn from_message<S: Into<String>>(msg: S) -> Self {
        PhotoInsightError {
            message: msg.into(),
        }
    }
}

impl fmt::Display for PhotoInsightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl std::error::Error for PhotoInsightError {}
