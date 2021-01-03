//! Generates [`String`] combinations from combinable [`Generator`]s.
//!
//! `generator-combinator` aims to provide text _generation_ capabilities, similar to but in the opposite direction of [parser combinators](https://en.wikipedia.org/wiki/Parser_combinator) that transform text into structured data.
//!
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

mod generator;
pub use generator::Generator;

mod value_generator;
pub use value_generator::ValueGenerator;
