use generator_combinator::{oneof, Generator};

/// Shows how `visit_one` is used
fn main() {
    let address = {
        let dir = oneof!("N", "E", "S", "W").optional();
        let num = Generator::Digit * (3, 4);
        let st = oneof!("Street", "Road", "Place");
        dir + ' ' + num + ' ' + st
    };

    let mut parts = Vec::new();
    address.visit_one(123456, |part| parts.push(part.to_string()));

    // NB: `num` produces repeated digits, so '91' is really '9' and '1'
    assert_eq!(parts, ["N", " ", "9", "1", " ", "Street"]);
}
