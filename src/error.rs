use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ParserError {

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Format error: {0}")]
    Format(String),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("Parse tickers file error: {0}")]
    ParseTickersFile(String)
}