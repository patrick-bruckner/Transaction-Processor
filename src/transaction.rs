use crate::types::*;

use serde::Deserialize;

/// Struct representing a transaction
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    typ: TransactionType,
    client: ClientID,
    #[serde(rename = "tx")]
    id: TransactionID,
    amount: Option<f64>,
    #[serde(skip)]
    in_dispute: bool
}

/// Different types of transactions
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

impl Transaction {
    /// Create a new deposit transaction from the provided info
    #[allow(dead_code)]
    pub fn new_deposit(client: ClientID, id: TransactionID, amount: f64,
                       in_dispute: bool) -> Self {
        Self {
            typ: TransactionType::Deposit,
            client,
            id,
            amount: Some(amount),
            in_dispute
        }
    }

    /// Create a new withdrawl transaction from the provided info
    #[allow(dead_code)]
    pub fn new_withdrawl(client: ClientID, id: TransactionID, amount: f64,
                       in_dispute: bool) -> Self {
        Self {
            typ: TransactionType::Withdrawal,
            client,
            id,
            amount: Some(amount),
            in_dispute
        }
    }

    /// Create a new dispute transaction from the provided info
    #[allow(dead_code)]
    pub fn new_dispute(client: ClientID, id: TransactionID) -> Self {
        Self {
            typ: TransactionType::Dispute,
            client,
            id,
            amount: None,
            in_dispute: false
        }
    }

    /// Create a new resolve transaction from the provided info
    #[allow(dead_code)]
    pub fn new_resolve(client: ClientID, id: TransactionID) -> Self {
        Self {
            typ: TransactionType::Resolve,
            client,
            id,
            amount: None,
            in_dispute: false
        }
    }

    /// Create a new chargeback transaction from the provided info
    #[allow(dead_code)]
    pub fn new_chargeback(client: ClientID, id: TransactionID) -> Self {
        Self {
            typ: TransactionType::Chargeback,
            client,
            id,
            amount: None,
            in_dispute: false
        }
    }

    pub fn validate(&self) -> bool {
        use TransactionType::*;
        match self.typ {
            Deposit | Withdrawal => {
                return self.amount.is_some();
            },
            Dispute | Resolve | Chargeback => {
                return (self.amount.is_none()) &&
                       (!self.in_dispute);
            }
        }
    }

    /// Get the type fo the transaction
    pub fn get_type(&self) -> TransactionType {
        self.typ
    }

    /// Get the Client ID associated with the transaction
    pub fn get_client_id(&self) -> ClientID {
        self.client
    }

    /// Get the transaction ID
    pub fn get_id(&self) -> TransactionID {
        self.id
    }

    /// Get the transaction ammout
    ///
    /// Note: Not all transactions types have an ammount
    pub fn get_amount(&self) -> Option<f64> {
        self.amount
    }

    /// Get the dispute status of the transaction
    pub fn is_disputed(&self) -> bool {
        self.in_dispute
    }

    /// Mark a transaction as disputed
    ///
    /// Note: Only deposits and withdrawals can be marked as disputed
    pub fn set_disputed(&mut self) {
        use TransactionType::*;
        match self.typ {
            Deposit | Withdrawal => self.in_dispute = true,
            _ => ()
        }
    }

    /// clear dispute status on a transaction
    pub fn clear_disputed(&mut self) {
        use TransactionType::*;
        match self.typ {
            Deposit | Withdrawal => self.in_dispute = false,
            _ => ()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_deposit() {
        let t = Transaction::new_deposit(500,600,100.0,false);

        assert_eq!(t.typ,TransactionType::Deposit);
        assert_eq!(t.client,500);
        assert_eq!(t.id,600);
        assert_eq!(t.amount,Some(100.0));
        assert_eq!(t.in_dispute,false);
        assert!(t.validate());
    }

    #[test]
    fn new_withdrawl() {
        let t = Transaction::new_withdrawl(500,600,100.0,true);

        assert_eq!(t.typ,TransactionType::Withdrawal);
        assert_eq!(t.client,500);
        assert_eq!(t.id,600);
        assert_eq!(t.amount,Some(100.0));
        assert_eq!(t.in_dispute,true);
        assert!(t.validate());
    }

    #[test]
    fn new_dispute() {
        let t = Transaction::new_dispute(500,600);

        assert_eq!(t.typ,TransactionType::Dispute);
        assert_eq!(t.client,500);
        assert_eq!(t.id,600);
        assert_eq!(t.amount,None);
        assert_eq!(t.in_dispute,false);
        assert!(t.validate());
    }

    #[test]
    fn new_resolve() {
        let t = Transaction::new_resolve(500,600);

        assert_eq!(t.typ,TransactionType::Resolve);
        assert_eq!(t.client,500);
        assert_eq!(t.id,600);
        assert_eq!(t.amount,None);
        assert_eq!(t.in_dispute,false);
        assert!(t.validate());
    }

    #[test]
    fn new_chargeback() {
        let t = Transaction::new_chargeback(500,600);

        assert_eq!(t.typ,TransactionType::Chargeback);
        assert_eq!(t.client,500);
        assert_eq!(t.id,600);
        assert_eq!(t.amount,None);
        assert_eq!(t.in_dispute,false);
        assert!(t.validate());
    }

    #[test]
    fn fail_validate() {
        let t1 = Transaction {
            typ: TransactionType::Deposit,
            client: 500,
            id: 600,
            amount: None,
            in_dispute: false
        };
        let t2 = Transaction {
            typ: TransactionType::Dispute,
            client: 500,
            id: 600,
            amount: Some(100.0),
            in_dispute: true
        };

        assert_eq!(t1.validate(),false);
        assert_eq!(t2.validate(),false);
    }

    #[test]
    fn set_disputed() {
        let mut t = Transaction::new_deposit(500,600,100.0,false);

        assert_eq!(t.in_dispute,false);
        assert!(!t.is_disputed());
        t.set_disputed();
        assert!(t.in_dispute);
        assert!(t.is_disputed());
    }

    #[test]
    fn clear_disputed() {
        let mut t = Transaction::new_deposit(500,600,100.0,true);

        assert!(t.in_dispute);
        assert!(t.is_disputed());
        t.clear_disputed();
        assert_eq!(t.in_dispute,false);
        assert!(!t.is_disputed());
    }

    #[test]
    fn getters() {
        let t = Transaction::new_deposit(500,600,100.0,false);

        assert_eq!(t.get_type(), t.typ);
        assert_eq!(t.get_client_id(), t.client);
        assert_eq!(t.get_id(), t.id);
        assert_eq!(t.get_amount(), t.amount);
        assert_eq!(t.is_disputed(), t.in_dispute);
    }
}
