use crossterm::ErrorKind;

#[derive(Debug)]
pub enum AppError {
    CrossTermError(ErrorKind),
}

impl From<ErrorKind> for AppError {
    fn from(f: ErrorKind) -> Self {
        AppError::CrossTermError(f)
    }
}
