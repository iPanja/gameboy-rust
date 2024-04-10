use std::fs::File;
use std::io::Read;

fn main() {
    // Read boot ROM file
    let mut buffer: Vec<u8> = Vec::new();

    let mut file = File::open("./DMG_ROM.bin").expect("INVALID ROM");
    file.read_to_end(&mut buffer).unwrap();

    //println!("{:?}", buffer);
    for &byte in buffer.iter() {
        println!("{:#X}", byte);
    }
}
