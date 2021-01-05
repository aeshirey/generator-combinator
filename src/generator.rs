#![allow(non_camel_case_types)]
use crate::ValueGenerator;
use std::{
    fmt::Display,
    ops::{Add, BitOr, Mul},
};

/// The building block of generator-combinators.
///
/// A `Generator` can be constructed from strings, chars, and slices:
///
/// ```
/// use generator_combinator::Generator;
/// let foo = Generator::from("foo"); // generates the string `foo`
/// let dot = Generator::from('.'); // generates the string `.`
/// let countries = Generator::from(&["US", "FR", "NZ", "CH"][..]); // generates strings `US`, `FR`, `NZ`, and `CH`.
/// ```
///
/// Individual `Generator`s can be combined as sequences with `+`, as variants with `|`, and with repetition with `* usize` and `* (usize, usize)`:
///
/// ```
/// use generator_combinator::Generator;
/// let foo = Generator::from("foo");
/// let bar = Generator::from("bar");
/// let foobar = foo.clone() + bar.clone(); // generates `foobar`
/// let foo_or_bar = foo.clone() | bar.clone(); // generates `foo`, `bar`
/// let foo_or_bar_x2 = foo_or_bar.clone() * 2; // generates `foofoo`, `foobar`, `barfoo`, `barbar`
/// let foo_x2_to_x4 = foo.clone() * (2, 4); // generates `foofoo`, `foofoofoo`, `foofoofoofoo`
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Generator {
    // Some convenience 'constants':
    /// Lowercase letters (a-z)
    AlphaLower,

    /// Uppercase letters (A-Z)
    AlphaUpper,

    /// Base-10 digits (0-9)
    Digit,

    /// Lowercase letters and digits (a-z0-9)
    AlphaNumLower,

    /// Uppercase letters and digits (A-Z0-9)
    AlphaNumUpper,

    /// Uppercase hexadecimal values (0-9A-F)
    HexUpper,

    /// Lowercase hexadecimal values (0-9a-f)
    HexLower,

    /// Generates a [`char`] literal.
    Char(char),

    /// Generates a [`String`] literal.
    ///
    /// Note that this is not a character class.
    /// `Str("foo".into())` generates the exact string `"foo"`
    Str(String),

    /// A choice between two or more patterns
    ///
    /// As a regex, this would be, eg, `(a|b|c)`
    OneOf(Vec<Generator>),

    /// An optional pattern, the same as [`RepeatedMN`](Self::RepeatedMN)`(a, 0, 1)`
    ///
    /// As a regex, this would be `a?`
    Optional(Box<Generator>),

    /// A pattern repeated exactly _n_ times. This is the same as [`RepeatedMN`](Self::RepeatedMN)`(a, n, n)`
    ///
    /// As a regex, this would be `a{n}`
    RepeatedN(Box<Generator>, usize),

    /// A pattern repeated at least _m_ times, as many as _n_ times.
    ///
    /// As a regex, this would be `a{m,n}`
    RepeatedMN(Box<Generator>, usize, usize),

    /// Two or more sequential patterns.
    ///
    /// As a regex, this would be, eg, `abc`
    Sequence(Vec<Generator>),
}

impl Generator {
    const ASCII_LOWER_A: u8 = 97;
    const ASCII_UPPER_A: u8 = 65;
    const ASCII_0: u8 = 48;

    /// Indicates whether this `Generator` covers multiple values.
    ///
    /// This is currently only used to identify whether [`regex`] should group in parens.
    fn is_multi(&self) -> bool {
        use Generator::*;
        match self {
            AlphaLower | AlphaUpper | Digit | AlphaNumUpper | AlphaNumLower | HexUpper
            | HexLower => false,
            Char(_) | Str(_) => false,
            OneOf(_) => true,
            Optional(_) => false,
            RepeatedN(_, _) => false,
            RepeatedMN(_, _, _) => false,
            Sequence(_) => true,
        }
    }

    /// Create a regular expression that represents the patterns generated.
    ///
    /// The result here is currently best-guess. It's not guaranteed valid, correct, idiomatic, etc.
    pub fn regex(&self) -> String {
        use Generator::*;

        match self {
            AlphaLower => "[a-z]".into(),
            AlphaUpper => "[A-Z]".into(),
            Digit => "\\d".into(),
            AlphaNumUpper => "[A-Z\\d]".into(),
            AlphaNumLower => "[a-z\\d]".into(),
            HexUpper => "[\\dA-F]".into(),
            HexLower => "[\\da-f]".into(),
            Char(c) => match c {
                &'.' => "\\.".into(),
                c => String::from(*c),
            },
            Str(s) => s.replace(".", "\\."),
            OneOf(v) => {
                let regexes = v.iter().map(|a| a.regex()).collect::<Vec<_>>();
                format!("({})", regexes.join("|"))
            }
            Optional(a) => {
                if a.is_multi() {
                    format!("({})?", a.regex())
                } else {
                    format!("{}?", a.regex())
                }
            }
            RepeatedN(a, n) => a.regex() + &"{" + &n.to_string() + &"}",
            RepeatedMN(a, m, n) => a.regex() + &"{" + &m.to_string() + &"," + &n.to_string() + &"}",
            Sequence(v) => {
                let regexes = v.iter().map(|a| a.regex()).collect::<Vec<_>>();
                regexes.join("")
            }
        }
    }

    /// The number of possible patterns represented.
    pub fn len(&self) -> u128 {
        use Generator::*;
        match self {
            AlphaLower | AlphaUpper => 26,
            Digit => 10,
            AlphaNumUpper | AlphaNumLower => 36,
            HexUpper | HexLower => 16,

            Char(_) | Str(_) => 1,

            OneOf(v) => v.iter().map(|a| a.len()).sum(),

            // Optionals add one value (empty/null)
            Optional(a) => 1 + a.len(),

            // Repeated variants are like base-x numbers of length n, where x is the number of combinations for a.
            // RepeatedN is easy:
            RepeatedN(a, n) => a.len().pow(*n as u32),
            // RepeatedMN has to remove the lower 'bits'/'digits'
            RepeatedMN(a, m, n) => {
                let base = a.len();
                (*m..=*n).map(|i| base.pow(i as u32)).sum()
            }

            Sequence(v) => v.iter().map(|a| a.len()).fold(1, |acc, n| acc * n),
        }
    }

    /// Recursively generates the pattern encoded in `num`, appending values to the `result`.
    fn generate_on_top_of(&self, num: &mut u128, result: &mut String) {
        use Generator::*;

        match self {
            AlphaLower => {
                let i = (*num % 26) as u8;
                *num /= 26;
                let c: char = (Self::ASCII_LOWER_A + i).into();
                result.push(c);
            }
            AlphaUpper => {
                let i = (*num % 26) as u8;
                *num /= 26;
                let c: char = (Self::ASCII_UPPER_A + i).into();
                result.push(c);
            }
            Digit => {
                let i = (*num % 10) as u8;
                *num /= 10;
                let c: char = (Self::ASCII_0 + i).into();
                result.push(c);
            }
            AlphaNumUpper => {
                let i = (*num % 36) as u8;
                *num /= 36;
                let c: char = if i < 26 {
                    Self::ASCII_UPPER_A + i
                } else {
                    Self::ASCII_0 + i - 26
                }
                .into();
                result.push(c);
            }
            AlphaNumLower => {
                let i = (*num % 36) as u8;
                *num /= 36;
                let c: char = if i < 26 {
                    Self::ASCII_LOWER_A + i
                } else {
                    Self::ASCII_0 + i - 26
                }
                .into();
                result.push(c);
            }

            HexUpper => {
                let i = (*num % 16) as u8;
                *num /= 16;
                let c: char = if i < 10 {
                    Self::ASCII_0 + i
                } else {
                    Self::ASCII_UPPER_A + i - 10
                }
                .into();
                result.push(c);
            }

            HexLower => {
                let i = (*num % 16) as u8;
                *num /= 16;
                let c: char = if i < 10 {
                    Self::ASCII_0 + i
                } else {
                    Self::ASCII_LOWER_A + i - 10
                }
                .into();
                result.push(c);
            }

            Char(c) => {
                result.push(*c);
            }
            Str(s) => {
                result.push_str(s);
            }
            OneOf(v) => {
                let i = (*num % v.len() as u128) as usize;
                *num /= v.len() as u128;
                v[i as usize].generate_on_top_of(num, result)
            }
            Optional(a) => {
                let a_len = a.len();
                let i = *num % (a_len + 1);

                let new_num = *num / (a_len + 1);

                if i > 0 {
                    // Use the optional value. First, undo its effect.
                    *num -= 1;

                    a.generate_on_top_of(num, result);
                }

                *num = new_num;
            }
            RepeatedN(a, n) => {
                // Repeat this one exactly n times
                let mut parts = Vec::with_capacity(*n);
                for _ in 0..*n {
                    let mut r = String::new();
                    a.generate_on_top_of(num, &mut r);
                    parts.push(r);
                }
                parts.reverse();
                result.push_str(&parts.join(""));
            }
            RepeatedMN(a, m, n) => {
                let mut parts = Vec::with_capacity(n - m + 1);
                for _ in *m..=*n {
                    let mut r = String::new();
                    a.generate_on_top_of(num, &mut r);
                    parts.push(r);
                }
                parts.reverse();
                result.push_str(&parts.join(""));
            }
            Sequence(v) => {
                for a in v {
                    a.generate_on_top_of(num, result);
                }
            }
        }
    }

    pub fn generate_exact(&self, num: u128) -> String {
        let range = self.len();
        assert!(num < range);

        let mut num = num;

        // build up a single string
        let mut result = String::new();
        self.generate_on_top_of(&mut num, &mut result);
        result
    }

    /// Makes this `Generator` optional.
    ///
    /// As a regex, this is the `?` operator.
    pub fn optional(self) -> Self {
        match &self {
            Generator::Optional(_) => self,
            _ => Generator::Optional(Box::new(self)),
        }
    }

    /// Provides an iterator across all possible values for this `Generator`.
    pub fn values(&self) -> ValueGenerator {
        ValueGenerator {
            c: self,
            n: self.len(),
            i: 0,
        }
    }
}

impl From<char> for Generator {
    fn from(c: char) -> Self {
        Generator::Char(c)
    }
}
impl From<&str> for Generator {
    fn from(s: &str) -> Self {
        Generator::Str(s.to_string())
    }
}

impl<T> From<&[T]> for Generator
where
    T: AsRef<str> + Display,
{
    fn from(values: &[T]) -> Self {
        let v = values
            .iter()
            .map(|value| Generator::Str(value.to_string()))
            .collect();
        Generator::OneOf(v)
    }
}

impl BitOr for Generator {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Generator::*;
        match (self, rhs) {
            (OneOf(mut v1), OneOf(v2)) => {
                for c in v2 {
                    v1.push(c);
                }

                OneOf(v1)
            }
            (OneOf(mut v1), rhs) => {
                v1.push(rhs);
                OneOf(v1)
            }
            (lhs, OneOf(v2)) => {
                let mut v = vec![lhs];
                for c in v2 {
                    v.push(c);
                }
                OneOf(v)
            }

            (lhs, rhs) => {
                let v = vec![lhs, rhs];
                OneOf(v)
            }
        }
    }
}

/// Mul operator for exact repetitions.
///
/// The following expressions are equivalent:
/// ```
/// use generator_combinator::Generator;
/// let foostr = Generator::from("foofoo");
/// let foomul = Generator::from("foo") * 2;
/// let fooadd = Generator::from("foo") + Generator::from("foo");
/// ```
impl Mul<usize> for Generator {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        let lhs = Box::new(self);
        Generator::RepeatedN(lhs, rhs)
    }
}

/// Mul operator for repetitions between `m` and `n` (inclusive)
/// ```
/// use generator_combinator::Generator;
/// let foo_two_to_five_times = Generator::from("foo") * (2,5);
/// ```
impl Mul<(usize, usize)> for Generator {
    type Output = Self;

    fn mul(self, rhs: (usize, usize)) -> Self::Output {
        let (m, n) = rhs;
        assert!(m <= n);

        let lhs = Box::new(self);
        Generator::RepeatedMN(lhs, m, n)
    }
}

/// Add operator for exact repetitions.
///
/// The following expressions are equivalent:
/// ```
/// use generator_combinator::Generator;
/// let foostr = Generator::from("foofoo");
/// let foomul = Generator::from("foo") * 2;
/// let fooadd = Generator::from("foo") + Generator::from("foo");
/// ```
impl Add for Generator {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use Generator::*;
        match (self, rhs) {
            (Sequence(mut v1), Sequence(v2)) => {
                for c in v2 {
                    v1.push(c);
                }

                Sequence(v1)
            }
            (Sequence(mut v1), rhs) => {
                v1.push(rhs);
                Sequence(v1)
            }
            (lhs, Sequence(v2)) => {
                let mut v = vec![lhs];
                for c in v2 {
                    v.push(c);
                }
                Sequence(v)
            }

            (lhs, rhs) => {
                let v = vec![lhs, rhs];
                Sequence(v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gen, oneof};
    #[test]
    fn combinations_consts() {
        let eight_alphas = Generator::AlphaLower * 8;
        assert_eq!(26u128.pow(8), eight_alphas.len());

        // This is the same as above
        let eight_alphas = Generator::AlphaLower * (8, 8);
        assert_eq!(26u128.pow(8), eight_alphas.len());

        // This is all combinations of exactly seven or exactly eight alphas:
        // aaaaaaa, aaaaaab, ..., zzzzzzz, aaaaaaaa, ..., zzzzzzzz
        let expected = 26u128.pow(7) + 26u128.pow(8);
        let seven_or_eight_alphas = Generator::AlphaLower * (7, 8);
        assert_eq!(expected, seven_or_eight_alphas.len());
    }

    #[test]
    fn combinations_mn() {
        /*
        Given the regex [ab]{2,3}, we can enumerate this easily:
        aa, ab, ba, bb,
        aaa, aab, aba, abb, baa, bab, bba, bbb
        Total combinations is therefore 12
        */

        let ab23 = (Generator::from("a") | Generator::from("b")) * (2, 3);
        assert_eq!(12, ab23.len());
    }

    #[test]
    fn combinations_str() {
        let foo = Generator::from("foo");
        assert_eq!(1, foo.len());
    }

    #[test]
    fn combinations_oneof() {
        let foo = Generator::from("foo");
        let bar = Generator::from("bar");
        assert_eq!(1, foo.len());
        assert_eq!(1, bar.len());

        let foo_bar = foo | bar;
        assert_eq!(2, foo_bar.len());

        let baz = Generator::from("baz");
        assert_eq!(1, baz.len());
        let foo_bar_baz = foo_bar | baz;
        assert_eq!(3, foo_bar_baz.len());
    }

    #[test]
    fn combinations_optional() {
        let foo = Generator::from("foo");
        let bar = Generator::from("bar");
        let foo_bar = foo.clone() | bar;

        let opt_foo = Generator::Optional(Box::new(foo));
        assert_eq!(2, opt_foo.len());

        let opt_foo_bar = Generator::Optional(Box::new(foo_bar));
        assert_eq!(3, opt_foo_bar.len());
    }

    #[test]
    fn combinations_email() {
        use Generator::Char;
        let username = Generator::AlphaLower * (6, 8);
        let user_combos = 26u128.pow(6) + 26u128.pow(7) + 26u128.pow(8);
        assert_eq!(username.len(), user_combos);

        let tld = Generator::from("com")
            | Generator::from("net")
            | Generator::from("org")
            | Generator::from("edu")
            | Generator::from("gov");
        let tld_combos = 5;
        assert_eq!(tld.len(), tld_combos);

        let domain = Generator::AlphaLower * (1, 8) + Char('.') + tld;
        let domain_combos = (1..=8).map(|i| 26u128.pow(i)).sum::<u128>() * tld_combos;
        assert_eq!(domain.len(), domain_combos);

        let email = username + Char('@') + domain;
        assert_eq!(email.len(), domain_combos * user_combos);
    }

    #[test]
    fn generate_alpha1() {
        let alphas2 = Generator::AlphaLower * 2;
        let aa = alphas2.generate_exact(0);
        assert_eq!(aa, "aa");

        let onetwothree = (Generator::Digit * 10).generate_exact(123);
        assert_eq!(onetwothree, "0000000123");
    }

    #[test]
    fn generate_hex() {
        let hex = Generator::from("0x") + Generator::HexUpper * 8;

        assert_eq!(4_294_967_296, hex.len());

        assert_eq!(hex.generate_exact(3_735_928_559), "0xDEADBEEF");
        assert_eq!(hex.generate_exact(464_375_821), "0x1BADD00D");
    }

    #[test]
    fn simplify() {
        let foo_opt1 = gen!("foo").optional();
        let foo_opt1 = foo_opt1.optional(); // making an optional optional shouldn't change it

        let foo_opt2 = gen!("foo").optional();
        assert_eq!(foo_opt1, foo_opt2);
    }

    #[test]
    fn equality() {
        let foo1 = Generator::from("foo");
        let foo2 = gen!("foo");
        assert_eq!(foo1, foo2);

        let foo2 = oneof!("foo");
        assert_eq!(foo1, foo2);
    }
}
