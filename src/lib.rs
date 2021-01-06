//! Generates [`String`] combinations from combinable [`Generator`]s.
//!
//! `generator-combinator` aims to provide text _generation_ capabilities, similar to but in the opposite direction of [parser combinators](https://en.wikipedia.org/wiki/Parser_combinator) that transform text into structured data.
//!
//! # Iris example
//! Consider the [regular expression](https://en.wikipedia.org/wiki/Regular_expression) `iris( (versicolor|virginica|setosa))?`. This regex matches exactly four input values:
//! * `"iris"`
//! * `"iris versicolor"`
//! * `"iris virginica"`
//! * `"iris setosa"`
//!
//! This regex does _not_ match other values such as `"iris fulva"` or `"iris "` (with trailing space). If we want to generate these four valid input values (something like a reverse regex), we can build a generator-combinator as:

//! ```
//! use generator_combinator::Generator;
//! let genus = Generator::from("iris");
//!
//! // Support three different species
//! let species = Generator::from("versicolor")
//!    | Generator::from("virginica")
//!    | Generator::from("setosa");
//! // Alternately:
//! let species = oneof!("versicolor", "virginica", "setosa");
//!
//! // Delimit the genus and species with a space
//! let species = Generator::from(' ') + species;
//!
//! // Allow generated values to be genus-only or with the species
//! let iris = genus + species.optional();
//!
//! // Our generator should produce exactly four values
//! assert_eq!(iris.len(), 4);
//!
//! let mut iris_values = iris.values();
//! assert_eq!(iris_values.next(), Some("iris".into()));
//! assert_eq!(iris_values.next(), Some("iris versicolor".into()));
//! assert_eq!(iris_values.next(), Some("iris virginica".into()));
//! assert_eq!(iris_values.next(), Some("iris setosa".into()));
//! assert_eq!(iris_values.next(), None);
//!
//! assert_eq!(iris.regex(), "iris( (versicolor|virginica|setosa))?");
//! ```
//!
//! # Street address example
//! Generators can be used to produce sample input data according to some pattern. For example, to generate street addresses (which aren't necessarily verifiable):
//! ```
//! use generator_combinator::{Generator, oneof, gen};
//! let space = Generator::from(' ');
//!
//! let number = Generator::Digit * (3, 5);
//!
//! let directional = space.clone() + oneof!("N", "E", "S", "W", "NE", "SE", "SW", "NW");
//! let street_names = space.clone() + oneof!("Boren", "Olive", "Spring", "Cherry", "Seneca", "Yesler", "Madison", "James", "Union", "Mercer");
//! let street_suffixes = space.clone() + oneof!("Rd", "St", "Ave", "Blvd", "Ln", "Dr", "Way", "Ct", "Pl");
//!
//! let address = number
//!     + directional.clone().optional()
//!     + street_names
//!     + street_suffixes
//!     + directional.clone().optional();
//!
//! assert_eq!(address.len(), 809_190_000);
//!
//! let addr_values = address.values();
//! println!("Example: {}", addr_values.random()); //Example: 344 W Yesler Way
//! println!("Example: {}", addr_values.random()); //Example: 702 NE Spring Ct N
//! println!("Example: {}", addr_values.random()); //Example: 803 SW Madison Way SE

mod macros;

mod generator;
pub use generator::Generator;

mod value_generator;
pub use value_generator::ValueGenerator;
