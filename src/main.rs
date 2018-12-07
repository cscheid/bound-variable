#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;

use std::vec::Vec;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use byteorder::ReadBytesExt;
use byteorder::BigEndian;
use std::io::Cursor;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use rocket::State;
use std::sync::Mutex;

mod um;

struct CommandSender(Mutex<Sender<um::Command>>);

#[post("/save/<file_name>")]
fn save(file_name: String, command_sender: State<CommandSender>) -> String
{
    let r = command_sender.0.lock().unwrap();
    r.send(um::Command::SaveState(file_name.clone())).unwrap();
    format!("Saved to {}", file_name)
}

#[post("/load/<file_name>")]
fn load(file_name: String, command_sender: State<CommandSender>) -> String
{
    let r = command_sender.0.lock().unwrap();
    r.send(um::Command::LoadState(file_name.clone())).unwrap();
    format!("Loaded from {}", file_name)
}

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
    file.read_to_end(&mut buffer_u8).expect("file should be readable");

    let mut buffer = vec![0; buffer_u8.len() / 4]; // Vec::new();
    let mut rdr = Cursor::new(buffer_u8);
    rdr.read_u32_into::<BigEndian>(&mut buffer).expect("buffer should be readable");

    let (command_sender, command_receiver) = channel();

    let machine_thread = thread::spawn(move || {
        let mut machine = um::init(buffer, command_receiver);
        machine.run();
    });

    let _rocket_thread = thread::spawn(move || {
        rocket::ignite()
            .mount("/", routes![save, load])
            .manage(CommandSender(Mutex::new(command_sender)))
            .launch();
    });
                                     
    machine_thread.join().unwrap();
}
