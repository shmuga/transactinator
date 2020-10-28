use rust_decimal::prelude::Decimal;

pub type TransactionId = u32;

// Trait is used to defined how some transaction would be applied to the customers account.
// Depending on the type of transaction we could increase/decrease available and hold values
pub trait ApplicableTransaction {
    fn to_hold(&self) -> Decimal {
        Decimal::new(0, 0)
    }

    fn tid(&self) -> TransactionId;
    fn to_add(&self) -> Decimal;
}

// Seperate structures for deposit and withdraw allow us to handle which transaction could be
// disputed on the type system level. Only deposit transaction implements ToDisputable trait for
// now.
#[derive(Copy, Clone, Debug)]
pub struct DepositTransaction {
    pub tid: TransactionId,
    amount: Decimal,
}

#[derive(Copy, Clone, Debug)]
pub struct WithdrawTransaction {
    pub tid: TransactionId,
    amount: Decimal,
}

#[derive(Copy, Clone, Debug)]
pub enum AmountTransaction {
    Deposit(DepositTransaction),
    Withdraw(WithdrawTransaction),
}

impl AmountTransaction {
    pub fn deposit(tid: TransactionId, amount: Decimal) -> AmountTransaction {
        AmountTransaction::Deposit(DepositTransaction { tid, amount })
    }

    pub fn withdraw(tid: TransactionId, amount: Decimal) -> AmountTransaction {
        AmountTransaction::Withdraw(WithdrawTransaction { tid, amount })
    }
}

impl ApplicableTransaction for AmountTransaction {
    fn tid(&self) -> TransactionId {
        match self {
            AmountTransaction::Deposit(t) => t.tid,
            AmountTransaction::Withdraw(t) => t.tid,
        }
    }
    fn to_add(&self) -> Decimal {
        match self {
            AmountTransaction::Deposit(t) => t.amount,
            AmountTransaction::Withdraw(t) => -t.amount, // in case of withdraw the sum should be deducted
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transaction {
    amount: Decimal,
}

#[derive(Copy, Clone, Debug)]
pub enum DisputableTransaction {
    Initiated(Transaction),
    Resolved(Transaction),
    ChargedBacked(Transaction),
}

impl DisputableTransaction {
    pub fn to_resolved(&self) -> Option<DisputableTransaction> {
        match self {
            // we should only resolve open disputes
            DisputableTransaction::Initiated(t) => Some(DisputableTransaction::Resolved(*t)),
            _ => None,
        }
    }

    pub fn to_chargebacked(&self) -> Option<DisputableTransaction> {
        match self {
            // we should only chargeback open disputes
            DisputableTransaction::Initiated(t) => Some(DisputableTransaction::ChargedBacked(*t)),
            _ => None,
        }
    }
}

impl ApplicableTransaction for DisputableTransaction {
    fn tid(&self) -> TransactionId { 0 }
    fn to_hold(&self) -> Decimal {
        match self {
            DisputableTransaction::Initiated(t) => t.amount,
            DisputableTransaction::Resolved(t) => -t.amount,
            DisputableTransaction::ChargedBacked(t) => -t.amount,
        }
    }

    fn to_add(&self) -> Decimal {
        match self {
            DisputableTransaction::Initiated(t) => -t.amount,
            DisputableTransaction::Resolved(t) => t.amount,
            DisputableTransaction::ChargedBacked(_) => Decimal::new(0, 0),
        }
    }
}

pub trait ToDisputable {
    fn to_disputable(&self) -> DisputableTransaction;
}

impl ToDisputable for DepositTransaction {
    fn to_disputable(&self) -> DisputableTransaction {
        DisputableTransaction::Initiated(Transaction {
            amount: self.amount,
        })
    }
}
