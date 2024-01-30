// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use bitfield::*;

#[bitfield]
pub struct RedirectionTableEntry {
    acknowledged: bool,
    trigger_mode: TriggerMode,
    delivery_mode: DeliveryMode,
    reserved: B3,
}

#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub enum TriggerMode {
    Edge = 0,
    Level = 1,
}

#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub enum DeliveryMode {
    Fixed = 0b000,
    Lowest = 0b001,
    SMI = 0b010,
    RemoteRead = 0b011,
    NMI = 0b100,
    Init = 0b101,
    Startup = 0b110,
    External = 0b111,
}

fn main() {



    println!("mag1: {}, mag2: {}", 2usize.pow(TriggerMode::BITS as u32)-1, TriggerMode::BITS2);
    println!("a: mag1: {}, mag2: {}", 2usize.pow(DeliveryMode::BITS as u32)-1, DeliveryMode::BITS2);

}

