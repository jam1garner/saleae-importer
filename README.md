# saleae-importer

A library for reading and writing Saleae Logic 2 binary capture data

### Example

```rust
use saleae_importer::SaleaeExport;

let data = SaleaeExport::open("digital_0.bin").unwrap();

for (is_high, time_len) in data.assume_digital().iter_samples() {
    println!("bit state: {is_high} | time: {time_len}");
}
```
