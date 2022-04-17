use crate::Generator;

/// Provides iterable access to the range of values represented by the [`Generator`]
pub struct StringIter<'a> {
    /// The generator to be used
    c: &'a Generator,

    /// The total number of values for `c`, equal to `c.len()`
    n: u128,

    /// The current value of the iterator. The first value is 0, the last is `n-1`.
    i: u128,
}

impl<'a> Iterator for StringIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.n {
            None
        } else {
            self.i += 1;
            Some(self.c.generate_one(self.i - 1))
        }
    }
}

#[cfg(feature = "with_rand")]
impl<'a> StringIter<'a> {
    /// Generates a random value in the [`Generator`]'s domain
    pub fn random(&self) -> String {
        let num = rand::random::<u128>() % self.n;
        self.c.generate_one(num)
    }
}

impl<'a> From<&'a Generator> for StringIter<'a> {
    fn from(c: &'a Generator) -> Self {
        Self {
            c,
            n: c.len(),
            i: 0,
        }
    }
}
