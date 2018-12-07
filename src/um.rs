extern crate serde_derive;
extern crate serde;
extern crate bincode;

use std::vec::Vec;
use std::io;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use bincode::serialize_into;
use bincode::deserialize_from;
use std::io::BufWriter;
use std::io::BufReader;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::mpsc::Receiver;

// http://www.boundvariable.org/um-spec.txt

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    SaveState(String),
    LoadState(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MachineState {
    count: usize,
    registers: [u32; 8],
    arrays: Vec<Vec<u32>>,
    array_list: Vec<u32>,
    execution_finger: usize,
    halted: bool
}

pub struct Machine {
    state: MachineState,
    command_receiver: Receiver<Command>
}

impl Machine {
    fn alloc(&mut self, n: u32) -> u32 {
        if self.state.array_list.len() > 0 {
            let ix = self.state.array_list.pop().unwrap();
            self.state.arrays[ix as usize].append(&mut vec![0; n as usize]);
            ix as u32
        } else {
            let ix = self.state.arrays.len();
            self.state.arrays.push(vec![0; n as usize]);
            ix as u32
        }
    }

    fn free(&mut self, a: u32) {
        self.state.arrays[a as usize].truncate(0);
        self.state.array_list.push(a);
    }
    pub fn snapshot(&self, filename: &String) {
        let mut f = BufWriter::new(File::create(filename).unwrap());
        serialize_into(&mut f, &self.state).unwrap();
    }
    pub fn load_from_snapshot(&mut self, filename: &String) {
        let f = BufReader::new(File::open(filename).unwrap());
        self.state = deserialize_from(f).unwrap();
    }
    pub fn run(&mut self) {
        while !self.state.halted {
            self.step();
        }
    }
    pub fn process_commands(&mut self) -> () {
        let v = self.command_receiver.try_recv();
        match v {
            Ok(Command::SaveState(filename)) => {
                println!("Will save! {}", &filename);
                self.snapshot(&filename);
            },
            Ok(Command::LoadState(filename)) => {
                println!("Will load! {}", &filename);
                self.load_from_snapshot(&filename);
            },
            Err(_) => {}
        }
    }

    pub fn step(&mut self) {
        self.state.count += 1;
        let current_platter = self.state.arrays[0][self.state.execution_finger];
        self.state.execution_finger += 1;
        let current_operator = current_platter >> 28;
        let reg_c = (current_platter & 7) as usize;
        let reg_b = ((current_platter >> 3) & 7) as usize;
        let reg_a = ((current_platter >> 6) & 7) as usize;
        match current_operator {
            0  => {
                if self.state.registers[reg_c] != 0 {
                    self.state.registers[reg_a] = self.state.registers[reg_b];
                };
            },
            1  => {
                let b = self.state.registers[reg_b] as usize;
                let c = self.state.registers[reg_c] as usize;
                self.state.registers[reg_a] = self.state.arrays[b][c];
            },
            2  => {
                let a = self.state.registers[reg_a] as usize;
                let b = self.state.registers[reg_b] as usize;
                let c = self.state.registers[reg_c];
                self.state.arrays[a][b] = c;
            },
            3  => {
                self.state.registers[reg_a] = self.state.registers[reg_b].wrapping_add(self.state.registers[reg_c]);
            },
            4  => {
                self.state.registers[reg_a] = self.state.registers[reg_b].wrapping_mul(self.state.registers[reg_c]);
            },
            5  => {
                self.state.registers[reg_a] = self.state.registers[reg_b].wrapping_div(self.state.registers[reg_c]);
            },
            6  => {
                self.state.registers[reg_a] = !(self.state.registers[reg_b] & self.state.registers[reg_c]);
            },
            7  => {
                self.state.halted = true;
                eprintln!("Halted after {:?} instructions", self.state.count);
            },
            8  => {
                self.state.registers[reg_b] = self.alloc(self.state.registers[reg_c]);
            },
            9  => {
                self.free(self.state.registers[reg_c]);
            },
            10 => {
                let mut stdout = io::stdout();
                {
                    let mut handle = stdout.lock();
                    let mut buffer = [0; 1];
                    buffer[0] = self.state.registers[reg_c] as u8;
                    handle.write(&buffer).expect("stdout should be writable");
                }
                stdout.flush().expect("stdout should be flushable");
                // println!("{:?}", self.state.registers[reg_c] as u8);
            },
            11 => {
                let (io_sender, io_receiver) = channel();
                let _io_thread = thread::spawn(move || {
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    let mut buffer = [0; 1];
                    let v = match handle.read(&mut buffer) {
                        Ok(_) =>  buffer[0].into(),
                        Err(_) => !0
                    };
                    io_sender.send(v).unwrap();
                });
                let mut done = false;
                while !done {
                    let io_result = io_receiver.try_recv();
                    match io_result {
                        Ok(v) => {
                            self.state.registers[reg_c] = v;
                            done = true;
                        },
                        Err(_) => {}
                    }
                    self.process_commands();
                }
            },
            12 => {
                let v = self.state.registers[reg_b];
                self.state.execution_finger = self.state.registers[reg_c] as usize;
                let b = self.state.registers[reg_b] as usize;
                if v != 0 {
                    self.state.arrays[0].truncate(0);
                    let mut a = self.state.arrays[b].clone();
                    self.state.arrays[0].append(&mut a);
                }
            },
            13 => {
                let imd_reg = (current_platter >> 25) & 7;
                let imd_value = current_platter & ((1 << 25) - 1);
                self.state.registers[imd_reg as usize] = imd_value;
            },
            _ => {}
        };
    }
}

pub fn init(program: Vec<u32>, command_receiver: Receiver<Command>) -> Machine {
    let arrays = vec![program];
    Machine {
        state: MachineState {
            count: 0,
            registers: [0; 8],
            arrays: arrays,
            array_list: Vec::new(),
            execution_finger: 0,
            halted: false
        },
        command_receiver: command_receiver
    }
}
