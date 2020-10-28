use crate::account::Account;
use crate::account::ClientId;
use crate::errors::TransactionError;
use crate::transaction::{AmountTransaction, TransactionId};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;

#[derive(Debug, Deserialize)]
struct Input(String, ClientId, TransactionId, Option<Decimal>);

type Accounts = HashMap<ClientId, Account>;

pub fn write(accounts: &Accounts) -> Result<(), csv::Error> {
    let mut writer = csv::Writer::from_writer(io::stdout());

    writer.write_record(&["client", "available", "held", "total", "locked"])?;

    for key in accounts.keys().sorted() {
        writer.write_record(&accounts[key].serialize())?;
    }

    writer.flush()?;
    Ok(())
}

pub fn run(file: &str) -> Result<(), csv::Error> {
    let file = File::open(file)?;

    let mut reader = csv::Reader::from_reader(file);

    let mut accounts = Accounts::new();

    for result in reader.deserialize() {
        let parsed: Input = result?;
        let Input(ttype, client, tx, amount) = parsed;

        let account = accounts.entry(client).or_insert(Account::new(client));
        let result = match ttype.as_ref() {
            "deposit" => account.process_transaction(&AmountTransaction::deposit(tx, amount.unwrap())),
            "withdraw" => account.process_transaction(&AmountTransaction::withdraw(tx, amount.unwrap())),
            "dispute" => account.open_dispute(tx),
            "resolve" => account.resolve(tx),
            "chargeback" => account.chargeback(tx),
            _ => Err(TransactionError::new(
                tx,
                &ttype,
                "Unknown transaction type",
            )),
        };

        result
            .map_err(|e| eprintln!("{}", e))
            .unwrap_or(()); // to not lose error and print it
    }

    write(&accounts)?;

    Ok(())
}
