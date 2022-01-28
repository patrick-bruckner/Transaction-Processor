# Transaction Processor
A toy transaction processor written in Rust. This program takes a CSV formatted list of transactions, processes those transactions, and outputs a CSV formatted list of client account balances and statuses.

## Building
`cargo build [--release]`

## Running
`cargo run [--release] -- <input file>`

### Example
Running `cargo run --release -- sample_input/in.csv` will output
```
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

## Run Tests
`cargo test`
