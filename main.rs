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
    d: B25,
    p2: B4,
}


fn main() {

    let mut bit_field = MyFourBytes::new();
    
    for i in 0..=0b1 {
        bit_field.set_a(i);

        //println!("{} == {}", i, bit_field.get_a());

        assert_eq!(i, bit_field.get_a());
    } 

    for i in 0..=0b111 {
        bit_field.set_b(i);

        //println!("{} == {}", i, bit_field.get_b());


        assert_eq!(i, bit_field.get_b());
    } 

    for i in 0..=0b1111 {
        bit_field.set_c(i);

        assert_eq!(i, bit_field.get_c());
    }

    bit_field.set_a(0);
    bit_field.set_b(0);
    bit_field.set_c(0);


    //[..ZA_BCDE], [FGGG_GGGH], [IJJJ_JJJK], [LMM._....]
    //to
    //[...._...Z], [ABCD_EFGG], [GGGG_HIJJ], [JJJJ_KLMM]
    
    /*const TEST: u32 = 0b1_1111_1111_1111_1111;
    bit_field.set_d(TEST);

    println!("data_arr: {}", bit_field.data.iter().map(|a| format!("[0b{:08b}]", a)).collect::<Vec<String>>().join(", "));

    assert_eq!(TEST, bit_field.get_d());*/
    for i in 0..=0x1_FF_FF_FF {
        bit_field.set_d(i);

        //println!("i {i}");
        //println!("data_arr: {}", bit_field.data.iter().map(|a| format!("[0b{:08b}]", a)).collect::<Vec<String>>().join(", "));

        assert_eq!(i, bit_field.get_d());
    } 


}


/*pub struct Test{
    data: [u8; 4]
}

impl Test{
    pub fn get_a(&self) -> <B1 as ::bitfield::Specifier>::Ty {
        const PREV_BITS: usize = 0;
        const START_BYTES: usize = PREV_BITS / 8;
        const START_BITS: usize = PREV_BITS % 8;
        const LAST_BITS: usize = PREV_BITS + <B1 as ::bitfield::Specifier>::BITS;
        const END_BYTES: usize = LAST_BITS / 8;
        const END_BITS: usize = LAST_BITS % 8;


        let mut arr = [0; (<B1 as ::bitfield::Specifier>::BITS + 7)/8];

        if START_BYTES == END_BYTES {
            const START_MASK: u8 = (0b1111_1111u8.overflowing_shr(START_BYTES as u32)).0;
            const END_MASK: u8 = (0b1111_1111u8.overflowing_shl((8 - END_BITS) as u32))
                .0;
            arr[0] = (self.data[START_BYTES] & START_MASK & END_MASK) >> (8 - END_BITS);

            return ByteArray(arr).into();
        }

        // 0            1           2
        //[...._AAAA],[BBBB_BBBB],[CCCC_CCCC]
        //to
        //[...._AAAA],[BBBB_BBBB],[CCCC_CCCC]

        if END_BITS == 0 {
            const START_MASK: u8 = 0b1111_1111u8.overflowing_shr(START_BITS as u32).0;

            arr[0] = self.data[START_BYTES] & START_MASK;

            #[allow(clippy::reversed_empty_ranges)]
            arr[..(END_BYTES - START_BYTES+1)].copy_from_slice(&self.data[START_BYTES+1..END_BYTES]);

            if START_BYTES != END_BYTES && END_BITS != 0 {
                arr[END_BYTES-START_BYTES] = self.data[END_BYTES].overflowing_shr((8 - END_BITS) as u32).0;
            }
            return ByteArray(arr).into();
        }


        // 0            1           2
        //[...._ABCD],[EFFF_FFFG],[HIII_J...]
        //to
        //[...._...A],[BCDE_FFFF],[FFGH_IIIJ]

        //[...._ABCD] -> [...._A...] requires mask 0000_1000
        //made with 0000_1111 & 1111_1000
        const START_MASK: u8 = 0b1111_1111u8.overflowing_shr(START_BITS as u32).0
            & 0b1111_1111u8.overflowing_shl((8-END_BITS) as u32).0;

        //takes [...._ABCD] -> [...._A...] -> [...._...A]
        arr[0] = (self.data[START_BYTES] & START_MASK).overflowing_shr((8 - END_BITS) as u32).0;

        #[allow(clippy::reversed_empty_ranges)]
        for byte in START_BYTES+1..=END_BYTES {
            //[...._ABCD] -> [...._.BCD] requires mask 0000_0111
            const A_MASK: u8 = 0b1111_1111u8.overflowing_shr((8 - END_BYTES) as u32).0;

            //[EFFF_FFFG] -> [EFFF_F...] requires mask 1111_1000
            const B_MASK: u8 = 0b1111_1111u8.overflowing_shl((8 - END_BITS) as u32).0;

            
            //[...._ABCD] | [EFFF_FFFG] -> [...._.BCD] | [EFFF_F...]
            //  -> [BCD._....] | [...E_FFFF] -> [BCDE_FFFF]
            //[...._ABCD] -> [...._.BCD] -> [BCD._....]
            arr[byte-START_BYTES] = (self.data[byte-1] & A_MASK).overflowing_shl(END_BITS as u32).0
                //[EFFF_FFFG] -> [EFFF_F...] -> [...E_FFFF]
                | (self.data[byte] & B_MASK).overflowing_shr((8 - END_BITS) as u32).0;
        }

        ByteArray(arr).into()

    }
}*/