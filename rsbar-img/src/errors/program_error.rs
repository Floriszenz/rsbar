use std::{
    error::Error,
    fmt,
    process::{ExitCode, Termination},
};

use image::ImageError;

pub type ProgramResult<T> = std::result::Result<T, ProgramError>;

#[derive(Debug)]
pub enum ProgramError {
    NoImagePassed,
    ProcessorInitFailed,
    ConfigParseFailed(String),
    ConfigSetFailed(String),
    ImageOpenFailed(ImageError),
    ImageProcessFailed(String),
    ImageScanFailed(String),
    NoSymbolDetected,
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
            ProgramError::ImageOpenFailed(..) => {
                write!(f, "Failed to open the image")
            }
            ProgramError::ImageProcessFailed(image_path) => {
                write!(f, "Failed to process the image \"{image_path}\"")
            }
            ProgramError::ImageScanFailed(image_path) => {
                write!(f, "Failed to scan the image \"{image_path}\"")
            }
            ProgramError::NoSymbolDetected => write!(f, "No symbol detected"),
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
            ProgramError::ImageOpenFailed(ref e) => Some(e),
            ProgramError::ImageProcessFailed(..) => None,
            ProgramError::ImageScanFailed(..) => None,
            ProgramError::NoSymbolDetected => None,
        }
    }
}

impl Termination for ProgramError {
    fn report(self) -> ExitCode {
        match self {
            ProgramError::NoSymbolDetected => ExitCode::from(4),
            _ => ExitCode::FAILURE,
        }
    }
}

impl From<ImageError> for ProgramError {
    fn from(error: ImageError) -> Self {
        ProgramError::ImageOpenFailed(error)
    }
}
