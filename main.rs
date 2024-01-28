// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use bitfield::*;

#[bitfield]
pub struct MyFourBytes {
    a: B1,
    b: B3,
    c: B4,
    p1: B3,
    d: B24,
    p2: B6,
}

/*pub struct MyFourBytes {
    data: [u8; 4]
}*/



fn main() {
    let mut bitfield = MyFourBytes::new();
    //assert_eq!(0, bitfield.get_a());
    //assert_eq!(0, bitfield.get_b());
    //assert_eq!(0, bitfield.get_c());
    //assert_eq!(0, bitfield.get_d());

    bitfield.set_d(0b1001_1011__1101_0111__1110_1101);

    println!("[{}]", bitfield.data.iter().map(|a| format!("0b{:08b}", a)).collect::<Vec<String>>().join(", "));

    bitfield.data[0] = 0b1111_0100;

    bitfield.data[1] = 0b1111_1001;
    bitfield.data[2] = 0b1010_1111;
    bitfield.data[3] = 0b1111_1001;

    println!("0b{:01b}", bitfield.get_a());
    println!("0b{:03b}", bitfield.get_b());
    println!("0b{:04b}", bitfield.get_c());

    println!("0b{:024b}", bitfield.get_d());
}
