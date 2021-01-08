use crate::Generator;

/// Provides iterable access to the range of values represented by the [`Generator`]
pub struct Iter<'a> {
    pub(crate) c: &'a Generator,
    pub(crate) n: u128,
    pub(crate) i: u128,
}

impl<'a> Iterator for Iter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.n {
            None
        } else {
            self.i += 1;
            Some(self.c.generate_exact(self.i - 1))
        }
    }
}

//#[cfg(with_rand)]
impl<'a> Iter<'a> {
    /// Generates a random value in the [`Generator`]'s domain
    pub fn random(&self) -> String {
        let num = rand::random::<u128>() % self.n;
        self.c.generate_exact(num)
    }
}
