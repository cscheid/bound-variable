extern crate byteorder;

use std::vec::Vec;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, BigEndian};
use std::io::Cursor;

mod um;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let display = path.display();
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };

    // let mut buffer = Vec::new();
    // match file.read_u32_into::<BigEndian>(&mut buffer) {
    //     Err(why) => {
    //         panic!("couldn't read: {}", why.description());
    //     },
    //     Ok(_) => {}
    // };
    // println!("{:?}", buffer.len());


    // seems inefficient, but shrug
    let mut buffer_u8 = Vec::new();
    file.read_to_end(&mut buffer_u8);

    let mut buffer = vec![0; buffer_u8.len() / 4]; // Vec::new();
    let mut rdr = Cursor::new(buffer_u8);
    rdr.read_u32_into::<BigEndian>(&mut buffer);

    let mut machine = um::init(buffer);
    machine.run();
}
