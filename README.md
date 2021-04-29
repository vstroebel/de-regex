# De-Regex

This crate contains a library that deserializes a string into a struct based on a regular expression and serde.

## Example

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Dimensions {
    width: u32,
    height: u32
}

let pattern = r"^(?P<width>\d+)x(?P<height>\d+)$";
let input = "800x600";

let dim: Dimensions = de_regexp::from_str(input, pattern).unwrap();

assert_eq!(dim.width, 800);
assert_eq!(dim.height, 600);
```

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in serde_urlencoded by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.