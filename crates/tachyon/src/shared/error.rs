use matrix_sdk::{ClientBuildError, HttpError};
use thiserror::Error;
#[derive(Error, Debug)]
pub enum TachyonError {

    #[error(transparent)]
    MatrixConversion(#[from] MatrixConversionError),
    #[error(transparent)]
    MatrixError(#[from] matrix_sdk::Error),
    #[error(transparent)]
    HttpError(#[from] HttpError),
    #[error(transparent)]
    ClientBuildError(#[from] ClientBuildError),
    #[error(transparent)]
    Any(#[from] anyhow::Error)
}

#[derive(Error, Debug)]
pub enum MatrixConversionError {
    #[error("Could not convert Email to Matrix ID: {}", .email)]
    EmailToMatrixId {email: String, source: anyhow::Error},
    #[error("Could not generate Device Id")]
    DeviceIdGeneration { source: anyhow::Error}

}