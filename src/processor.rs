use crate::types::*;
use crate::client::Client;
use crate::transaction::Transaction;

use std::collections::HashMap;
use std::io;

use csv::{Error,ReaderBuilder,Trim};

/// The main struct of the Transaction Processor
pub struct TransactionProcessor {
    clients: HashMap<ClientID,Client>,
    transactions: HashMap<TransactionID,Transaction>
}

/// Transaction Processor Error
#[derive(Debug)]
pub enum TransactionProcessorErr {
    CSVError(Error),
    TransactionValidateError(String)
}

impl TransactionProcessor {
    /// Create a new TransactionProcessor
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            transactions: HashMap::new()
        }
    }

    /// Process a list of CSV formatted transactions
    pub fn process_csv_stream<R>(&mut self, reader: R) -> Result<(),TransactionProcessorErr>
            where R: io::Read {
        use TransactionProcessorErr::*;

        let mut csv_reader = ReaderBuilder::new()
            .trim(Trim::All)    // allow leading/trailing whitespace
            .from_reader(reader);
        for raw_trans in csv_reader.deserialize() {
            let trans: Transaction = raw_trans.map_err(|e| CSVError(e))?;
            // validate transaction since it's possible an invalid one
            //  was formed
            if !trans.validate() {
                return Err(TransactionValidateError(format!("{:?}",trans)));
            }
            self.process_transaction(trans);
        }

        Ok(())
    }

    /// Process a single transaction
    ///
    /// Note: A client will be created if one does not already exist
    pub fn process_transaction(&mut self, trans: Transaction) {
        // add client if client doesn't exist
        if let None = self.clients.get(&trans.get_client_id()) {
            let client = Client::new(trans.get_client_id());
            self.clients.insert(trans.get_client_id(), client);
        }

        // we just added the client if it didn't exist so unwrap shouldn't
        //  panic here
        let client = self.clients.get_mut(&trans.get_client_id()).unwrap();

        // within this match calls to get_amount are unwraped because we know
        //  at those times that it is Some bacause of where the transaction
        //  came from or what type of transaction it is
        use crate::transaction::TransactionType::*;
        match trans.get_type() {
            // add funds to client and record transaction
            Deposit => {
                client.add_funds(trans.get_amount().unwrap());
                self.transactions.insert(trans.get_id(), trans);
            },
            // remove funds from client and record transaction if remove was
            //  possible
            Withdrawal => {
                if client.remove_funds(trans.get_amount().unwrap()) {
                    self.transactions.insert(trans.get_id(), trans);
                }
            },
            // if disputed transaction was found hold funds from client
            Dispute => {
                if let Some(trans_other) = self.transactions.get_mut(&trans.get_id()) {
                    client.hold_funds(trans_other.get_amount().unwrap());
                    trans_other.set_disputed();
                }
            },
            // if disputed transaction was found and is in dispute
            //  restore held funds to client
            Resolve => {
                if let Some(trans_other) = self.transactions.get_mut(&trans.get_id()) {
                    if trans_other.is_disputed() {
                        client.restore_funds(trans_other.get_amount().unwrap());
                        trans_other.clear_disputed();
                    }
                }
            },
            // if disputed transaction was found and is in dispute
            //  remove held function from client and lock client
            Chargeback => {
                if let Some(trans_other) = self.transactions.get_mut(&trans.get_id()) {
                    if trans_other.is_disputed() {
                        client.restore_funds(trans_other.get_amount().unwrap());
                        client.remove_funds(trans_other.get_amount().unwrap());
                        client.lock();
                        trans_other.clear_disputed();
                    }
                }
            }
        };
    }

    /// Export Client info in CSV format
    pub fn write_csv_to_stream<W>(&self, writer: W) -> Result<(),TransactionProcessorErr>
            where W: io::Write {
        use TransactionProcessorErr::*;

        let mut csv_writer = csv::Writer::from_writer(writer);

        for c in self.clients.values() {
            csv_writer.serialize(c).map_err(|e| CSVError(e))?;
        }

        Ok(())
    }

    /// Print Client list to stdout
    #[allow(dead_code)]
    pub fn print_clients(&self) {
        for c in self.clients.values() {
            println!("{:?}",c);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::client::*;
    use crate::transaction::*;

    #[test]
    fn in_out_basic() {
        let input =
            "type, client, tx, amount\n\
             deposit, 1, 1, 1.0\n\
             deposit, 1, 3, 2.0\n\
             withdrawal, 1, 4, 1.5";
        let expected_out =
            "client,available,held,total,locked\n\
             1,1.5000,0.0000,1.5000,false\n";

        let mut out_buf = Vec::new();
        let mut tp = TransactionProcessor::new();
        tp.process_csv_stream(input.as_bytes()).unwrap();
        tp.write_csv_to_stream(&mut out_buf).unwrap();
        let out = std::str::from_utf8(out_buf.as_slice()).unwrap().to_string();

        assert_eq!(out, expected_out);
    }

    #[test]
    fn in_out_resolve() {
        let input =
            "type, client, tx, amount\n\
             deposit, 1, 1, 100.0\n\
             dispute, 1, 1,\n\
             resolve, 1, 1,";
        let expected_out =
            "client,available,held,total,locked\n\
             1,100.0000,0.0000,100.0000,false\n";

        let mut out_buf = Vec::new();
        let mut tp = TransactionProcessor::new();
        tp.process_csv_stream(input.as_bytes()).unwrap();
        tp.write_csv_to_stream(&mut out_buf).unwrap();
        let out = std::str::from_utf8(out_buf.as_slice()).unwrap().to_string();

        assert_eq!(out, expected_out);
    }

    #[test]
    fn in_out_chargeback() {
        let input =
            "type, client, tx, amount\n\
             deposit, 1, 1, 100.0\n\
             dispute, 1, 1,\n\
             chargeback, 1, 1,";
        let expected_out =
            "client,available,held,total,locked\n\
             1,0.0000,0.0000,0.0000,true\n";

        let mut out_buf = Vec::new();
        let mut tp = TransactionProcessor::new();
        tp.process_csv_stream(input.as_bytes()).unwrap();
        tp.write_csv_to_stream(&mut out_buf).unwrap();
        let out = std::str::from_utf8(out_buf.as_slice()).unwrap().to_string();

        assert_eq!(out, expected_out);
    }

    #[test]
    fn bad_input1() {
        let input =
            "type, client, tx, amount\n\
             deposit, 1, 1, 1.0\n\
             deposit, 1, 3,\n\
             withdrawal, 1, 4, 1.5";

        let mut tp = TransactionProcessor::new();
        let result = tp.process_csv_stream(input.as_bytes()).unwrap_err();
        match result {
            TransactionProcessorErr::TransactionValidateError(_) => (),
            _ => panic!("incorrect result")
        }
    }

    #[test]
    fn bad_input2() {
        let input =
            "type, client, tx, amount\n\
             deposit, 1, 1, 1.0\n\
             deposit,\n\
             withdrawal, 1, 4, 1.5";

        let mut tp = TransactionProcessor::new();
        let result = tp.process_csv_stream(input.as_bytes()).unwrap_err();
        match result {
            TransactionProcessorErr::CSVError(_) => (),
            _ => panic!("incorrect result")
        }
    }

    #[test]
    fn deposit() {
        let c_id = 500;
        let t_id = 600;
        let amount = 100.0;

        let mut tp = TransactionProcessor::new();
        let t = Transaction::new_deposit(c_id, t_id, amount, false);

        tp.process_transaction(t);

        let c = tp.clients.get(&c_id);
        assert!(c.is_some());   // ensure client was created
        assert_eq!(c.unwrap().get_available_funds(),amount);
        assert!(tp.transactions.get(&t_id).is_some());
    }

    #[test]
    fn withdrawal() {
        let c_id = 500;
        let t_id = 600;
        let amount = 100.0;
        let wothdraw_amount = 10.0;

        let mut tp = TransactionProcessor::new();
        let mut c = Client::new(c_id);
        c.add_funds(amount);
        tp.clients.insert(c_id, c);

        let t = Transaction::new_withdrawl(c_id, t_id, wothdraw_amount, false);

        tp.process_transaction(t);

        let ec = tp.clients.get(&c_id).unwrap();
        assert_eq!(ec.get_available_funds(),amount-wothdraw_amount);
    }

    #[test]
    fn dispute() {
        let c_id = 500;
        let t_id = 600;
        let amount = 100.0;

        let mut tp = TransactionProcessor::new();
        let c = Client::new(c_id);
        tp.clients.insert(c_id, c);

        let t1 = Transaction::new_deposit(c_id, t_id, amount, false);
        let t2 = Transaction::new_dispute(c_id, t_id);

        tp.process_transaction(t1);
        tp.process_transaction(t2);

        let ec = tp.clients.get(&c_id).unwrap();
        assert_eq!(ec.get_held_funds(),amount);

        let et = tp.transactions.get(&t_id).unwrap();
        assert!(et.is_disputed());
    }

    #[test]
    fn resolve() {
        let c_id = 500;
        let t_id = 600;
        let amount = 100.0;

        let mut tp = TransactionProcessor::new();
        let c = Client::new(c_id);
        tp.clients.insert(c_id, c);

        let t1 = Transaction::new_deposit(c_id, t_id, amount, false);
        let t2 = Transaction::new_dispute(c_id, t_id);
        let t3 = Transaction::new_resolve(c_id, t_id);

        tp.process_transaction(t1);
        tp.process_transaction(t2);
        tp.process_transaction(t3);

        let ec = tp.clients.get(&c_id).unwrap();
        assert_eq!(ec.get_available_funds(),amount);

        let et = tp.transactions.get(&t_id).unwrap();
        assert!(!et.is_disputed());
    }

    #[test]
    fn chargeback() {
        let c_id = 500;
        let t_id = 600;
        let amount = 100.0;

        let mut tp = TransactionProcessor::new();
        let c = Client::new(c_id);
        tp.clients.insert(c_id, c);

        let t1 = Transaction::new_deposit(c_id, t_id, amount, false);
        let t2 = Transaction::new_dispute(c_id, t_id);
        let t3 = Transaction::new_chargeback(c_id, t_id);

        tp.process_transaction(t1);
        tp.process_transaction(t2);
        tp.process_transaction(t3);

        let ec = tp.clients.get(&c_id).unwrap();
        assert_eq!(ec.get_available_funds(),0.0);
        assert!(ec.is_locked());

        let et = tp.transactions.get(&t_id).unwrap();
        assert!(!et.is_disputed());
    }
}
