use serde::Serialize;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub struct Request<D> {
    pub event: String,
    pub data: D,
}
