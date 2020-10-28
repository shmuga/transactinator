use crate::transaction::TransactionId;
use std::fmt;

#[derive(Debug, Clone)]
pub struct TransactionError {
    tid: TransactionId,
    operation: String,
    message: String,
}

impl TransactionError {
    pub fn new<'a>(tid: TransactionId, operation: &'a str, message: &'a str) ->  TransactionError {
        TransactionError { tid, operation: operation.to_string(), message: message.to_string() }
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error processing transaction tid={}. {} operation failed. {}",
            self.tid, self.operation, self.message
        )
    }
}
