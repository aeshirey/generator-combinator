# generator-combinator
Provides [parser-combinator](https://en.wikipedia.org/wiki/Parser_combinator)-like combinable text _generation_ in Rust.

You can add this crate to your Rust project with `generator-combinator = "0.1.0"`. [Documentation on docs.rs](https://docs.rs/generator-combinator) and [crates.io listing](https://crates.io/crates/generator-combinator).

To generate street address-like input, only a few components are required. We can quickly produce a range of nearly 1B possible values that can be fully iterated over or randomly sampled:

```rust
use generator_combinator::Generator;
let space = Generator::from(' ');

let number = Generator::Digit * (3, 5);

let directional = space.clone() + oneof!("N", "E", "S", "W", "NE", "SE", "SW", "NW");
let street_names = space.clone() + oneof!("Boren", "Olive", "Spring", "Cherry", "Seneca", "Yesler", "Madison", "James", "Union", "Mercer");
let street_suffixes = space.clone() + oneof!("Rd", "St", "Ave", "Blvd", "Ln", "Dr", "Way", "Ct", "Pl");

let address = number // street number is 3-5 digits long
    + directional.clone().optional() // optional pre-directional
    + street_names
    + street_suffixes
    + directional.clone().optional(); // optional post-directional

assert_eq!(address.len(), 809_190_000);

let addr_values = address.values();
println!("Example: {}", addr_values.random()); //Example: 344 W Yesler Way
println!("Example: {}", addr_values.random()); //Example: 702 NE Spring Ct N
println!("Example: {}", addr_values.random()); //Example: 803 SW Madison Way SE
```

This library is 0.1.0 - there may be issues, functionality may be incomplete, etc. 

## Known issues / _nota bene_
- Generated digits include leading zeros

## TODO
- [ ] Consider including `Fn` variants of `Generator`
- [ ] `Generator` post-processing of component strings (eg, to strip leading zeros) before combining output
