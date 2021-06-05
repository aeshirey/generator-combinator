#![allow(non_camel_case_types)]
use crate::iter::StringIter;
use crate::transformfn::TransformFn;
use std::{
    fmt::Display,
    mem,
    ops::{Add, AddAssign, BitOr, BitOrAssign, Mul, MulAssign},
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
#[derive(Clone, Debug, PartialEq)]
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
    /// As a regex, this would be, eg, `(a|b|c)?` (depending on `is_optional`)
    OneOf {
        v: Vec<Generator>,
        is_optional: bool,
    },

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

    Transform {
        inner: Box<Generator>,
        transform_fn: TransformFn,
    },
}

impl Generator {
    const ASCII_LOWER_A: u8 = 97;
    const ASCII_UPPER_A: u8 = 65;
    const ASCII_0: u8 = 48;

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
            OneOf { v, is_optional } => {
                let regexes = v.iter().map(|a| a.regex()).collect::<Vec<_>>();
                let mut grp = format!("({})", regexes.join("|"));
                if *is_optional {
                    grp.push('?');
                }
                grp
            }
            RepeatedN(a, n) => a.regex() + &"{" + &n.to_string() + &"}",
            //RepeatedN(a, n) => format!("{}{{{}}}", a.regex, n),
            RepeatedMN(a, m, n) => a.regex() + "{" + &m.to_string() + "," + &n.to_string() + "}",
            Sequence(v) => {
                let regexes = v.iter().map(|a| a.regex()).collect::<Vec<_>>();
                regexes.join("")
            }
            Transform {
                inner,
                transform_fn: _,
            } => inner.regex(),
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

            OneOf { v, is_optional } => {
                // Optionals add one value (empty/null)
                v.iter().map(|a| a.len()).sum::<u128>() + if *is_optional { 1 } else { 0 }
            }

            // Repeated variants are like base-x numbers of length n, where x is the number of combinations for a.
            // RepeatedN is easy:
            RepeatedN(a, n) => a.len().pow(*n as u32),
            // RepeatedMN has to remove the lower 'bits'/'digits'
            RepeatedMN(a, m, n) => {
                let base = a.len();
                (*m..=*n).map(|i| base.pow(i as u32)).sum()
            }

            Sequence(v) => v.iter().map(|a| a.len()).product(),
            Transform {
                inner,
                transform_fn: _,
            } => inner.len(),
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
            OneOf { v, is_optional } => {
                let v_len = self.len();

                // Divide out the impact of this OneOf; the remainder can be
                // used internally and we'll update num for parent recursions.
                let new_num = *num / v_len;
                *num %= v_len;

                if *is_optional && *num == 0 {
                    // use the optional - don't recurse and don't update result
                } else {
                    if *is_optional {
                        *num -= 1;
                    }
                    for a in v {
                        let a_len = a.len() as u128;
                        if *num < a_len {
                            a.generate_on_top_of(num, result);
                            break;
                        } else {
                            // subtract out the impact of this OneOf branch
                            *num -= a_len;
                        }
                    }
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
            Transform {
                inner,
                transform_fn,
            } => {
                let mut r = String::new();
                inner.generate_on_top_of(num, &mut r);
                let r = (transform_fn.0)(r);
                result.push_str(&r);
            }
        }
    }

    /// Generates the [String] encoded by the specified `num`.
    ///
    /// Panics if `num` exceeds the length given by [Generator::len]
    pub fn generate_one(&self, num: u128) -> String {
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
        use Generator::OneOf;
        match self {
            OneOf {
                v,
                is_optional: true,
            } => OneOf {
                v,
                is_optional: true,
            },
            OneOf {
                v,
                is_optional: false,
            } => OneOf {
                v,
                is_optional: true,
            },
            _ => OneOf {
                v: vec![self],
                is_optional: true,
            },
        }
    }

    /// Provides an iterator across all possible values for this `Generator`.
    pub fn generate_all(&self) -> StringIter {
        self.into()
    }

    pub fn transform(self, f: fn(String) -> String) -> Self {
        let transform_fn = TransformFn(Box::new(f));

        Self::Transform {
            inner: Box::new(self),
            transform_fn,
        }
    }

    // Removes redundant [`Self::Optional`] values from [`Self::OneOf`].
    //
    // When `OneOf` contains multiple `Optional` values, the generator would incorrectly
    // produce two logically identical results. This function detects this scenario,
    // keeping the first `Optional` value and unboxing the remaining values, if any.
    /*
    fn reduce_optionals(v: &mut Vec<Generator>) {
        use Generator::*;
        let mut found_opt = false;

        for a in v.iter_mut() {
            if let Optional(inner) = a {
                if found_opt {
                    // convert this to non-optional
                    let inner = mem::replace(inner, Box::new(Digit));
                    *a = *inner;
                } else {
                    // first optional can be kept
                    found_opt = true;
                }
            }
        }
    }
    */

    //pub fn visit_all(&self) -> VisitIter {
    //    self.into()
    //}

    pub fn visit_one<F>(&self, mut num: u128, mut cb: F)
    where
        F: FnMut(&str),
    {
        let range = self.len();
        assert!(num < range);

        self.visit_exact_inner(&mut num, &mut cb);
    }

    fn visit_exact_inner<F>(&self, num: &mut u128, cb: &mut F)
    where
        F: FnMut(&str),
    {
        use Generator::*;

        match self {
            AlphaLower => {
                let i = (*num % 26) as u8;
                *num /= 26;
                let c: char = (Self::ASCII_LOWER_A + i).into();
                cb(&String::from(c));
            }
            AlphaUpper => {
                let i = (*num % 26) as u8;
                *num /= 26;
                let c: char = (Self::ASCII_UPPER_A + i).into();
                cb(&String::from(c));
            }
            Digit => {
                let i = (*num % 10) as u8;
                *num /= 10;
                let c: char = (Self::ASCII_0 + i).into();
                cb(&String::from(c));
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
                cb(&String::from(c));
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
                cb(&String::from(c));
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
                cb(&String::from(c));
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
                cb(&String::from(c));
            }
            Char(c) => cb(&String::from(*c)),
            Str(s) => cb(s),
            OneOf { v, is_optional } => {
                let v_len = self.len();

                // Divide out the impact of this OneOf; the remainder can be
                // used internally and we'll update num for parent recursions.
                let new_num = *num / v_len;
                *num %= v_len;

                if *is_optional && *num == 0 {
                    // use the optional - don't recurse and don't update result
                } else {
                    if *is_optional {
                        *num -= 1;
                    }
                    for a in v {
                        let a_len = a.len() as u128;
                        if *num < a_len {
                            a.visit_exact_inner(num, cb);
                            break;
                        } else {
                            // subtract out the impact of this OneOf branch
                            *num -= a_len;
                        }
                    }
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

                parts.iter().rev().for_each(|part| cb(part));
            }
            RepeatedMN(a, m, n) => {
                let mut parts = Vec::with_capacity(n - m + 1);
                for _ in *m..=*n {
                    let mut r = String::new();
                    a.generate_on_top_of(num, &mut r);
                    parts.push(r);
                }
                parts.iter().rev().for_each(|part| cb(part));
            }
            Sequence(v) => v.iter().for_each(|a| a.visit_exact_inner(num, cb)),
            Transform {
                inner,
                transform_fn,
            } => {
                let mut r = String::new();
                inner.generate_on_top_of(num, &mut r);
                let r = (transform_fn.0)(r);
                cb(&r);
            }
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
        // todo: check for & remove empty strings and set is_optional to true
        let is_optional = false;
        let v = values
            .iter()
            .map(|value| Generator::Str(value.to_string()))
            .collect();
        Generator::OneOf { v, is_optional }
    }
}

impl BitOr for Generator {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Generator::*;
        match (self, rhs) {
            (
                OneOf {
                    v: mut v1,
                    is_optional: opt1,
                },
                OneOf {
                    v: v2,
                    is_optional: opt2,
                },
            ) => {
                v1.extend(v2);

                let is_optional = opt1 || opt2;
                OneOf { v: v1, is_optional }
            }
            (OneOf { mut v, is_optional }, rhs) => {
                v.push(rhs);
                OneOf { v, is_optional }
            }
            (lhs, OneOf { mut v, is_optional }) => {
                v.insert(0, lhs);
                OneOf { v, is_optional }
            }

            (lhs, rhs) => {
                let v = vec![lhs, rhs];
                OneOf {
                    v,
                    is_optional: false,
                }
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

impl MulAssign<usize> for Generator {
    fn mul_assign(&mut self, rhs: usize) {
        let repeat = self.clone() * rhs;
        *self = repeat;
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

impl MulAssign<(usize, usize)> for Generator {
    fn mul_assign(&mut self, rhs: (usize, usize)) {
        let (m, n) = rhs;
        assert!(m <= n);

        // temporarily swap in Digit to avoid cloning self
        let lhs = mem::replace(self, Generator::Digit);
        *self = Generator::RepeatedMN(Box::new(lhs), m, n);
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

impl AddAssign for Generator {
    fn add_assign(&mut self, rhs: Self) {
        use Generator::*;
        match (self, rhs) {
            (Sequence(v1), Sequence(v2)) => {
                for c in v2 {
                    v1.push(c);
                }
            }
            (Sequence(v1), rhs) => {
                v1.push(rhs);
            }
            (lhs, Sequence(mut v2)) => {
                let left = mem::replace(lhs, Generator::Digit);
                v2.insert(0, left);
                *lhs = Sequence(v2);
            }

            (lhs, rhs) => {
                let left = mem::replace(lhs, Generator::Digit);
                let v = vec![left, rhs];
                *lhs = Sequence(v)
            }
        }
    }
}

impl BitOrAssign for Generator {
    fn bitor_assign(&mut self, rhs: Self) {
        use Generator::*;
        match (self, rhs) {
            (
                OneOf {
                    v: v1,
                    is_optional: opt1,
                },
                OneOf {
                    v: v2,
                    is_optional: opt2,
                },
            ) => {
                v1.push(Digit);
                v1.extend(v2);
                if opt2 {
                    *opt1 = true;
                }
            }
            (OneOf { v, is_optional: _ }, rhs) => {
                v.push(rhs);
            }
            (lhs, OneOf { mut v, is_optional }) => {
                // swap out left to avoid clone
                let left = mem::replace(lhs, Generator::Digit);
                v.insert(0, left);
                *lhs = OneOf { v, is_optional };
            }

            (lhs, rhs) => {
                let left = mem::replace(lhs, Generator::Digit);
                let v = vec![left, rhs];
                //Self::reduce_optionals(&mut v);
                *lhs = OneOf {
                    v,
                    is_optional: false,
                };
            }
        }

        // address potential duplicate optionals
        //let s = mem::replace(self, Digit);
        //*self = s.reduce_optionals();
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

        let opt_foo = Generator::OneOf {
            v: vec![foo.clone()],
            is_optional: true,
        };
        assert_eq!(2, opt_foo.len());

        let opt_foo_bar = Generator::OneOf {
            v: vec![foo.clone(), bar.clone()],
            is_optional: true,
        };
        assert_eq!(3, opt_foo_bar.len());

        let mut v = opt_foo_bar.generate_all();
        assert_eq!(Some("".into()), v.next());
        assert_eq!(Some("foo".into()), v.next());
        assert_eq!(Some("bar".into()), v.next());
        assert_eq!(None, v.next());
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
        let aa = alphas2.generate_one(0);
        assert_eq!(aa, "aa");

        let onetwothree = (Generator::Digit * 10).generate_one(123);
        assert_eq!(onetwothree, "0000000123");

        // Same thing but with postprocessing
        let onetwothree = (Generator::Digit * 10)
            .transform(|s| s.trim_start_matches('0').to_string())
            .generate_one(123);
        assert_eq!(onetwothree, "123");
    }

    #[test]
    fn generate_hex() {
        let hex = Generator::from("0x") + Generator::HexUpper * 8;

        assert_eq!(4_294_967_296, hex.len());

        assert_eq!(hex.generate_one(3_735_928_559), "0xDEADBEEF");
        assert_eq!(hex.generate_one(464_375_821), "0x1BADD00D");
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
        // test the macros
        let foo1 = Generator::from("foo");
        let foo2 = gen!("foo");
        assert_eq!(foo1, foo2);

        let foo2 = oneof!("foo");
        assert_eq!(foo1, foo2);

        // test BitOrAssign
        let foobar1 = oneof!("foo", "bar");
        let mut foobar2 = gen!("foo");
        foobar2 |= gen!("bar");
        assert_eq!(foobar1, foobar2);

        // test AddAssign
        let foobar1 = gen!("foo") + gen!("bar");
        let mut foobar2 = gen!("foo");
        foobar2 += gen!("bar");
        assert_eq!(foobar1, foobar2);

        // test MulAssign<usize>
        let foo1 = gen!("foo") * 2;
        let mut foo2 = gen!("foo");
        foo2 *= 2;
        assert_eq!(foo1, foo2);

        // test MulAssign<(usize,usize)>
        let foo1 = gen!("foo") * (2, 3);
        let mut foo2 = gen!("foo");
        foo2 *= (2, 3);
        assert_eq!(foo1, foo2);
    }

    #[test]
    fn test_reduce_optionals() {
        // A naive implementation might treat this as:
        // ("foo" | "") | ("bar" | "") | ("baz" | ""), which could incorrectly generate two unnecessary empty strings
        let foo = gen!("foo").optional();
        let bar = gen!("bar").optional();
        let baz = gen!("baz").optional();
        let foobarbaz1 = foo | bar | baz;

        // The ideal approach is to know that with each of foo, bar, and baz being optional, it's the same as:
        let foobarbaz2 = gen!("foo").optional() | oneof!("bar", "baz");

        // Which they are, taken care of by BitOr
        assert_eq!(foobarbaz1, foobarbaz2);

        // And it will generate the four values as expected
        let values: Vec<_> = foobarbaz1.generate_all().collect();
        assert_eq!(vec!["", "foo", "bar", "baz"], values);

        // Note that the optional value is boosted to the front of the line and foo|bar|baz are commoned up
        let foobarbaz3 = gen!("foo") | gen!("bar").optional() | gen!("baz");
        assert_eq!(foobarbaz1, foobarbaz3);
        assert!(
            matches!(foobarbaz3, Generator::OneOf { v, is_optional } if v.len() == 3 && is_optional)
        );
    }

    #[test]
    fn test_transform() {
        let foobarbaz = oneof!("foo", "bar", "baz");

        // Trim any leading 'b' from (foo|bar|baz)
        let fooaraz = foobarbaz.clone().transform(|s| {
            if s.starts_with("b") {
                s.trim_start_matches('b').to_string()
            } else {
                s
            }
        });

        assert_eq!(3, fooaraz.len());
        assert_eq!("foo", fooaraz.generate_one(0));
        assert_eq!("ar", fooaraz.generate_one(1));
        assert_eq!("az", fooaraz.generate_one(2));

        // Uppercase (foo|bar|baz)
        let foobarbaz_upper = foobarbaz.clone().transform(|s| s.to_uppercase());
        assert_eq!(3, foobarbaz_upper.len());
        assert_eq!("FOO", foobarbaz_upper.generate_one(0));
        assert_eq!("BAR", foobarbaz_upper.generate_one(1));
        assert_eq!("BAZ", foobarbaz_upper.generate_one(2));

        let ten_digits = Generator::Digit * 10;
        let onetwothree = ten_digits.generate_one(123);
        assert_eq!(onetwothree, "0000000123");
        let onetwothree = ten_digits
            .transform(|s| s.trim_start_matches('0').to_string())
            .generate_one(123);
        assert_eq!(onetwothree, "123");
    }

    #[test]
    fn test_visit() {
        let foobarbaz = oneof!("foo", "bar", "baz");
        let fbb_nnnn = foobarbaz + Generator::Digit * 4;

        let bar1234 = fbb_nnnn.generate_one(3703);
        assert_eq!("bar1234", bar1234);

        let mut s = String::with_capacity(7);
        fbb_nnnn.visit_one(3703, |part| s.push_str(part));
        assert_eq!("bar1234", s);
    }

    #[test]
    fn regex() {
        let foobarbaz = oneof!("foo", "bar", "baz");
        let fbb_nnnn = foobarbaz + Generator::Digit * 4;
        assert_eq!("(foo|bar|baz)\\d{4}", fbb_nnnn.regex());

        let hi45 = Generator::from("hi") * (4, 5);
        assert_eq!("hi{4,5}", hi45.regex());

        let sea = Generator::from("Seattle") + gen!(", WA").optional();
        assert_eq!("Seattle(, WA)?", sea.regex());
    }

    quickcheck! {
        /// Check that `generate_one` will produce the same string as would be visited.
        fn street_addresses(n: u128) -> bool {
            const RANGE : u128 = 809_190_000;

            let space = Generator::from(' ');
            let number = (Generator::Digit * (3, 5)).transform(|s| s.trim_start_matches('0').to_string());

            let directional = space.clone() + oneof!("N", "E", "S", "W", "NE", "SE", "SW", "NW");
            let street_names = space.clone() + oneof!("Boren", "Olive", "Spring", "Cherry", "Seneca", "Yesler", "Madison", "James", "Union", "Mercer");
            let street_suffixes = space.clone() + oneof!("Rd", "St", "Ave", "Blvd", "Ln", "Dr", "Way", "Ct", "Pl");

            let address = number
                + directional.clone().optional()
                + street_names
                + street_suffixes
                + directional.clone().optional();

            assert_eq!(address.len(), RANGE);
            let n = n % RANGE;

            let generated = address.generate_one(n);

            let mut visited  = String::with_capacity(generated.len());
            address.visit_one(n, |part| visited.push_str(part));
            assert_eq!(visited, generated);

            true
        }
    }
}
