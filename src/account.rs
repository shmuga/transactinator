use crate::errors::TransactionError;
use rust_decimal::prelude::*;
use std::collections::HashMap;

use crate::transaction::ToDisputable;
use crate::transaction::{
    AmountTransaction, ApplicableTransaction, DisputableTransaction, TransactionId,
};

pub type ClientId = u16;
#[derive(Debug)]
pub struct Account {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    locked: bool,
    transaction_log: HashMap<TransactionId, AmountTransaction>,
    open_disputes: HashMap<TransactionId, DisputableTransaction>,
}

const PRECISION: u32 = 4;

impl Account {
    pub fn new(client: ClientId) -> Account {
        Account {
            client,
            available: Decimal::new(0, 10),
            held: Decimal::new(0, 10),
            locked: false,
            transaction_log: HashMap::new(),
            open_disputes: HashMap::new(),
        }
    }

    pub fn serialize(&self) -> Vec<String> {
        let available = self.available.round_dp(PRECISION);
        let held = self.held.round_dp(PRECISION);
        let total = (available + held).round_dp(PRECISION);

        vec![
            self.client.to_string(),
            available.to_string(),
            held.to_string(),
            total.to_string(),
            self.locked.to_string(),
        ]
    }

    fn lock(&mut self) {
        self.locked = true;
    }

    // apply any type of tranaction only when there funds avilable for it
    fn apply<'a>(&mut self, transaction: &dyn ApplicableTransaction) -> Result<(), &'a str> {
        let available = self.available + transaction.to_add();

        if available >= Decimal::new(0, 0) {
            self.available = available;
            self.held += transaction.to_hold();
            Ok(())
        } else {
            Err("Not enough money to apply transaction")
        }
    }

    fn is_not_locked<'a>(&self) -> Result<(), &'a str> {
        // TODO: Refactor when bool_to_option is stable https://github.com/rust-lang/rust/issues/64260
        if !self.locked {
            Ok(())
        } else {
            Err("Account is locked")
        }
    }

    // Does check for
    // 1. Account locking
    // 2. Duplications of tranactions
    // 3. Available fund via .apply
    //
    // Logs the transaction in case of success.
    pub fn process_transaction(
        &mut self,
        transaction: &AmountTransaction,
    ) -> Result<(), TransactionError> {
        self.is_not_locked()
            .and_then(|_| match self.transaction_log.get(&transaction.tid()) {
                Some(_) => Err("Transaction is already processed"),
                _ => Ok(()),
            })
            .and_then(|_| self.apply(transaction))
            .map(|_| {
                self.transaction_log.insert(transaction.tid(), *transaction);
            })
            .map_err(|message| {
                TransactionError::new(transaction.tid(), "process_transaction", message)
            })
    }

    // Does check for
    // 1. Account locking
    // 2. Existance of the transaction
    // 3. If disput is not open already
    // 4. If transaction could be converted into dispute
    //
    // Stores all open disputes.
    pub fn open_dispute(&mut self, tid: TransactionId) -> Result<(), TransactionError> {
        self.is_not_locked()
            .and_then(|_| {
                self.transaction_log
                    .get(&tid)
                    .ok_or_else(|| "Missing transaction for the client")
            })
            .and_then(|transaction| {
                if let Some(_) = self.open_disputes.get(&tid) {
                    Err("Dispute is already open")
                } else {
                    Ok(transaction)
                }
            })
            .and_then(|transaction| {
                let disputable = match transaction {
                    AmountTransaction::Deposit(t) => Some(t.to_disputable()),
                    _ => None,
                };

                disputable.ok_or_else(|| "Incorrect type of transaction for dispute")
            })
            .and_then(|dispute| {
                let result = self.apply(&dispute);
                self.open_disputes.insert(tid, dispute);
                result
            })
            .map_err(|message| TransactionError::new(tid, "open_dispute", &message))
    }

    // Does check for
    // 1. Account locking
    // 2. Existance of the dispute
    // 3. If disput is valid for resolution
    //
    // Removes dispute after successful application of transaction
    pub fn resolve(&mut self, tid: TransactionId) -> Result<(), TransactionError> {
        self.is_not_locked()
            .and_then(|_| {
                self.open_disputes
                    .get(&tid)
                    .ok_or_else(|| "Missing dipsute for transaction")
            })
            .and_then(|dispute| {
                dispute
                    .to_resolved()
                    .ok_or_else(|| "Could not resolve such dispute")
            })
            .and_then(|resolved| {
                let result = self.apply(&resolved);
                self.open_disputes.remove(&tid);
                result
            })
            .map_err(|message| TransactionError::new(tid, "resolve", &message))
    }

    // Does check for
    // 1. Account locking
    // 2. Existance of the dispute
    // 3. If disput is valid for resolution
    //
    // Removes dispute after successful application of transaction and locks an account
    pub fn chargeback(&mut self, tid: TransactionId) -> Result<(), TransactionError> {
        self.is_not_locked()
            .and_then(|_| {
                self.open_disputes
                    .get(&tid)
                    .ok_or_else(|| "Missing dipsute for transaction")
            })
            .and_then(|dispute| {
                dispute
                    .to_chargebacked()
                    .ok_or_else(|| "Could not chargeback such dispute")
            })
            .and_then(|resolved| {
                let result = self.apply(&resolved);
                self.open_disputes.remove(&tid);
                self.lock();
                result
            })
            .map_err(|message| TransactionError::new(tid, "chargeback", &message))
    }
}
