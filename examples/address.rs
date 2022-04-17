use generator_combinator::{oneof, Generator};

fn main() {
    let dir = oneof!("N", "E", "S", "W").optional();
    let num = (Generator::Digit * (3, 4)).transform(|s| {
        // Not perfect -- this would generate "11st" instead of "11th"
        let last = s.chars().last().unwrap();
        s + match last {
            '1' => "st",
            '2' => "nd",
            '3' => "rd",
            _ => "th",
        }
    });
    let st = oneof!("Street", "Road", "Place");

    let address = dir + ' ' + num + ' ' + st;
    println!("address produces {} values", address.len());

    let samp = address.generate_one(123456);
    println!("Sample address: {samp}");
}
