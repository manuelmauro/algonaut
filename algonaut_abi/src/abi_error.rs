extern crate derive_more;
use derive_more::{Display, From};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Display, Error, From, Clone)]
pub enum AbiError {
    #[display("{_0}")]
    Msg(String),
}
