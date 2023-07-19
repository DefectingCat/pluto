use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlutoError {
    #[error("wrong arguments {0}")]
    ArgsError(&'static str),
}
