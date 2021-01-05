#[macro_export]
macro_rules! gen {
    ($a : expr) => {
        Generator::from($a)
    };
}

#[macro_export]
macro_rules! oneof {
    ($a : expr) => {
        Generator::from($a)
    };
    ($a:expr, $($b:expr),+) => {
        Generator::from($a) | oneof!($($b),+)
    };
}
