pub mod future_helper;
pub mod radio;
pub mod radio2;

/// An error struct that allows an error message to be reported along with additional information.
/// This is necessary to return an error, but we need to return other data, likely data that transferred
/// ownership to the function that failed.
pub struct ErrorPlus<T, E> {
    pub error: E,
    pub other: T,
}

// Implement the Display and Debug traits for ErrorPlus<T> so that the error message can be printed
// and proprogated up with ? in the fail condition.
impl<T, E> std::fmt::Display for ErrorPlus<T, E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl<T, E> std::fmt::Debug for ErrorPlus<T, E>
where
    E: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl<T, E> std::error::Error for ErrorPlus<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
