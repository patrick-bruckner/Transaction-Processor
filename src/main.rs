mod client;
mod processor;
mod transaction;
mod types;

use processor::TransactionProcessor;

use std::env;
use std::fs::File;
use std::io;

/// A Transaction Processor that's able to read from a CSV file of transactions
/// and write out a CSV list of Client account states after processing the
/// transactions
fn main() {
    if env::args().len() == 1 {
        panic!("Expected at least 1 arg -- a CSV file path");
    }

    let path = env::args().nth(1).unwrap();
    let csv_handle = File::open(path).unwrap();

    let mut tp = TransactionProcessor::new();
    tp.process_csv_stream(csv_handle).unwrap();
    tp.write_csv_to_stream(io::stdout()).unwrap();
}
