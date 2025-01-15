#[derive(Debug)]
pub enum Error {
    GetHistoryTxError(String),
    HttpRequestError,
    GetTokenTxError(String),
}

#[derive(Debug)]
pub enum TxParseError {
    InvalidField(String)
}
