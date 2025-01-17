# Currency
Converts a currency into another from the command line.

It uses Open Exchange Rates to fetch the latest rates.

## Usage
```
currency FROM TO amount
```
Exemple:
```
$ currency USD CNY 100
USD 100.0000 = CNY 732.9000
```
## Build
```
git clone https://github.com/topiga/currency
cd currency
cargo build --release
```
The executable will be in the ``./target/release/`` directory

## Credits

- ARClab for the [original C code](https://github.com/arclabch/currency/)
- Open Exchange Rates for their [API](https://openexchangerates.org/)
- Rust for their amazing language
- The devs of [reqwest](https://github.com/seanmonstar/reqwest), [serde and serde_json](https://serde.rs/), and [tokio](https://tokio.rs/)
