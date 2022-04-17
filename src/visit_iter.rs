use crate::Generator;

/// Provides iterable access to the range of values represented by the [`Generator`]
#[derive(Clone, Debug)]
pub struct VisitIter<'a> {
    c: &'a Generator,
    n: u128,
    i: u128,
}

impl<'a> VisitIter<'a> {
    pub fn visit<F>(&self, cb: F)
    where
        F: FnMut(&str),
    {
        self.c.visit_one(self.i, cb);
    }
}

impl<'a> Iterator for VisitIter<'a> {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.n {
            None
        } else {
            let result = self.clone();
            self.i += 1;
            Some(result)
        }
    }
}

#[cfg(feature = "with_rand")]
impl<'a> VisitIter<'a> {
    /// Generates a random value in the [`Generator`]'s domain
    pub fn random(&self) -> String {
        let num = rand::random::<u128>() % self.n;
        self.c.generate_one(num)
    }
}

impl<'a> From<&'a Generator> for VisitIter<'a> {
    fn from(c: &'a Generator) -> Self {
        Self {
            c,
            n: c.len(),
            i: 0,
        }
    }
}
