use std::{
    error::Error,
    fmt,
    process::{ExitCode, Termination},
};

pub type ProgramResult<T> = std::result::Result<T, ProgramError>;

#[derive(Debug)]
pub enum ProgramError {
    NoImagePassed,
    ProcessorInitFailed,
    ConfigParseFailed(String),
    ConfigSetFailed(String),
    ImageScanFailed(String),
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramError::NoImagePassed => write!(f, "Specify image file(s) to scan"),
            ProgramError::ProcessorInitFailed => write!(f, "Failed to initialize the processor"),
            ProgramError::ConfigParseFailed(setting) => {
                write!(f, "Failed to parse the config \"{setting}\"")
            }
            ProgramError::ConfigSetFailed(setting) => {
                write!(
                    f,
                    "Failed to set the config \"{setting}\" for the processor"
                )
            }
            ProgramError::ImageScanFailed(image_path) => {
                write!(f, "Failed to scan the image \"{image_path}\"")
            }
        }
    }
}

impl Error for ProgramError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ProgramError::NoImagePassed => None,
            ProgramError::ProcessorInitFailed => None,
            ProgramError::ConfigParseFailed(..) => None,
            ProgramError::ConfigSetFailed(..) => None,
            ProgramError::ImageScanFailed(..) => None,
        }
    }
}

impl Termination for ProgramError {
    fn report(self) -> ExitCode {
        ExitCode::FAILURE
    }
}
