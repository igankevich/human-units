/// Unit parsing error.
#[derive(Debug)]
pub struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
