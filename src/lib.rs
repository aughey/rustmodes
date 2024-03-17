pub mod future_helper;
pub mod radio;

/// An error struct that allows an error message to be reported along with additional information.
/// This is necessary to return an error, but we need to return other data, likely data that transferred
/// ownership to the function that failed.
pub struct ErrorPlus<T> {
    pub error: anyhow::Error,
    pub other: T,
}

// Implement the Display and Debug traits for ErrorPlus<T> so that the error message can be printed
// and proprogated up with ? in the fail condition.
impl<T> std::fmt::Display for ErrorPlus<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl<T> std::fmt::Debug for ErrorPlus<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl<T> std::error::Error for ErrorPlus<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
