use crate::types::*;

use serde::Serialize;

/// Struct representing a Client's info
#[derive(Debug, Serialize)]
pub struct Client {
    #[serde(rename = "client")]
    id: ClientID,
    #[serde(serialize_with = "serialize_f64_to_4")]
    available: f64,
    #[serde(serialize_with = "serialize_f64_to_4")]
    held: f64,
    #[serde(serialize_with = "serialize_f64_to_4")]
    total: f64,
    locked: bool
}

impl Client {
    /// Create a Client with a given ID
    pub fn new(id: ClientID) -> Self {
        Self {
            id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false
        }
    }

    /// Get a Client's ID
    #[allow(dead_code)]
    pub fn get_client_id(&self) -> ClientID {
        self.id
    }

    /// Get a Client's available funds
    #[allow(dead_code)]
    pub fn get_available_funds(&self) -> f64 {
        self.available
    }

    /// Get a Client's held funds
    #[allow(dead_code)]
    pub fn get_held_funds(&self) -> f64 {
        self.held
    }

    /// Get a Client's total funds
    ///
    /// total_funds == available + held
    #[allow(dead_code)]
    pub fn get_total_funds(&self) -> f64 {
        self.total
    }

    /// Get lock status of Client
    ///
    /// A client's balances can't be modified if the account is locked
    #[allow(dead_code)]
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Add funds to a Client's account
    ///
    /// Operation will fail if Client's account is locked
    pub fn add_funds(&mut self, amount: f64) -> bool {
        // only add funds if account isn't locked
        if !self.locked {
            self.available += amount;
            self.total += amount;

            true
        } else {
            false
        }
    }

    /// Remove funds from a Client's account
    ///
    /// Operation will fail if Client's account is locked or there are not
    /// sufficient available funds
    pub fn remove_funds(&mut self, amount: f64) -> bool {
        // only remove funds if account isn't locked and required
        //  funds are available
        if (self.available >= amount) && (!self.locked) {
            self.available -= amount;
            self.total -= amount;

            true
        } else {
            false
        }
    }

    /// Hold funds in a Client's account
    ///
    /// Operation will fail if Client's account is locked
    pub fn hold_funds(&mut self, amount: f64) -> bool {
        // only hold funds if account isn't locked
        if !self.locked {
            self.available -= amount;
            self.held += amount;

            true
        } else {
            false
        }
    }

    /// Restore held funds for a Client's account
    ///
    /// Operation will fail if Client's account is locked or there are not
    /// sufficient held funds
    pub fn restore_funds(&mut self, amount: f64) -> bool {
        // only restore funds if account isn't locked and required held
        //  funds are available
        if (self.held >= amount) && (!self.locked) {
            self.available += amount;
            self.held -= amount;

            true
        } else {
            false
        }
    }

    /// Lock a Client's account
    ///
    /// A client's balances can't be modified if the account is locked
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Unlock a Client's account
    #[allow(dead_code)]
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

fn serialize_f64_to_4<S>(data: &f64, s: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
    s.serialize_str(format!("{:.4}",data).as_str())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_funds() {
        let mut c = Client::new(500);
        let amount = 100.0;
        let add_amount = 10.0;

        c.available = amount;
        c.total = amount;

        assert!(c.add_funds(add_amount));

        assert_eq!(c.available, amount+add_amount);
        assert_eq!(c.total, amount+add_amount);
    }

    #[test]
    fn remove_funds_success() {
        let mut c = Client::new(500);
        let amount = 100.0;
        let remove_amount = 10.0;

        c.available = amount;
        c.total = amount;

        assert!(c.remove_funds(remove_amount));

        assert_eq!(c.available,amount-remove_amount);
        assert_eq!(c.total,amount-remove_amount);
    }

    #[test]
    fn remove_funds_fail() {
        let mut c = Client::new(500);
        let amount = 100.0;

        c.available = amount;
        c.total = amount;

        assert_eq!(c.remove_funds(amount+1.0),false);
        assert_eq!(c.available,amount);
        assert_eq!(c.total,amount);
    }

    #[test]
    fn hold_funds() {
        let mut c = Client::new(500);
        let amount = 100.0;
        let hold_amount = 10.0;

        c.available = amount;
        c.total = amount;

        assert!(c.hold_funds(hold_amount));
        assert_eq!(c.held,hold_amount);
        assert_eq!(c.available,amount-hold_amount);
        assert_eq!(c.total,amount);
    }

    #[test]
    fn restore_funds() {
        let mut c = Client::new(500);
        let amount = 100.0;
        let hold_amount = 10.0;

        c.available = amount-hold_amount;
        c.total = amount;
        c.held = hold_amount;

        assert!(c.restore_funds(hold_amount));
        assert_eq!(c.held,0.0);
        assert_eq!(c.available,amount);
        assert_eq!(c.total,amount);
    }

    #[test]
    fn getters() {
        let mut c = Client::new(500);
        let amount = 100.0;
        let hold_amount = 10.0;

        c.available = amount-hold_amount;
        c.total = amount;
        c.held = hold_amount;

        assert_eq!(c.get_held_funds(), c.held);
        assert_eq!(c.get_available_funds(), c.available);
        assert_eq!(c.get_total_funds(), c.total);
        assert_eq!(c.get_client_id(), c.id);
        assert_eq!(c.is_locked(), c.locked);
    }

    #[test]
    fn lock() {
        let mut c = Client::new(500);

        assert!(!c.locked);
        assert!(!c.is_locked());
        c.lock();
        assert!(c.locked);
        assert!(c.is_locked());
    }

    #[test]
    fn unlock() {
        let mut c = Client::new(500);

        c.lock();
        assert!(c.locked);
        assert!(c.is_locked());
        c.unlock();
        assert!(!c.locked);
        assert!(!c.is_locked());
    }
}
