use std::collections::HashMap;
use std::vec::Vec;
use std::io;
use std::io::Write;
use std::io::Read;

// http://www.boundvariable.org/um-spec.txt

pub struct Machine {
    count: usize,
    registers: [u32; 8],
    arrays: HashMap<u32, Vec<u32>>,
    array_list: Vec<u32>,
    execution_finger: usize,
    halted: bool
}

impl Machine {

    fn alloc(&mut self, n: u32) -> u32 {
        let ix = if self.array_list.len() > 0 {
            self.array_list.pop().unwrap()
        } else {
            self.arrays.len() as u32
        };
        self.arrays.insert(ix, vec![0; n as usize]);
        ix as u32
    }

    fn free(&mut self, a: u32) {
        self.arrays.remove(&a);
        self.array_list.push(a);
    }
    pub fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }
    pub fn step(&mut self) {
        let current_platter = self.arrays[&0][self.execution_finger];
        self.execution_finger += 1;
        let current_operator = current_platter >> 28;
        let reg_c = (current_platter & 7) as usize;
        let reg_b = ((current_platter >> 3) & 7) as usize;
        let reg_a = ((current_platter >> 6) & 7) as usize;
        match current_operator {
            0  => {
                if self.registers[reg_c] != 0 {
                    self.registers[reg_a] = self.registers[reg_b];
                };
            },
            1  => {
                self.registers[reg_a] = self.arrays[&self.registers[reg_b]][self.registers[reg_c] as usize];
            },
            2  => {
                let a = self.registers[reg_a];
                let b = self.registers[reg_b] as usize;
                let c = self.registers[reg_c];
                self.arrays
                    .entry(a as u32)
                    .and_modify(|e| e[b] = c);
            },
            3  => {
                self.registers[reg_a] = self.registers[reg_b].wrapping_add(self.registers[reg_c]);
            },
            4  => {
                self.registers[reg_a] = self.registers[reg_b].wrapping_mul(self.registers[reg_c]);
            },
            5  => {
                self.registers[reg_a] = self.registers[reg_b].wrapping_div(self.registers[reg_c]);
            },
            6  => {
                self.registers[reg_a] = !(self.registers[reg_b] & self.registers[reg_c]);
            },
            7  => {
                self.halted = true;
            },
            8  => {
                self.registers[reg_b] = self.alloc(self.registers[reg_c]);
            },
            9  => {
                self.free(self.registers[reg_c]);
            },
            10 => {
                let mut stdout = io::stderr();
                {
                    let mut handle = stdout.lock();
                    let mut buffer = [0; 1];
                    buffer[0] = self.registers[reg_c] as u8;
                    handle.write(&buffer);
                }
                stdout.flush();
                // println!("{:?}", self.registers[reg_c] as u8);
            },
            11 => {
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                let mut buffer = [0; 1];
                match handle.read(&mut buffer) {
                    Ok(_) => self.registers[reg_c] = buffer[0].into(),
                    Err(_) => self.registers[reg_c] = !0
                };
            },
            12 => {
                let v = self.registers[reg_b];
                self.execution_finger = self.registers[reg_c] as usize;
                if v != 0 {
                    self.arrays.get(&self.registers[reg_b])
                        .map(|a| a.clone())
                        .map(|a| {
                            self.arrays.remove(&0);
                            self.arrays.insert(0, a);
                        });
                }
            },
            13 => {
                let imd_reg = (current_platter >> 25) & 7;
                let imd_value = current_platter & ((1 << 25) - 1);
                self.registers[imd_reg as usize] = imd_value;
            },
            _ => {}
        };
    }
}

pub fn init(program: Vec<u32>) -> Machine {
    let mut arrays = HashMap::new();
    arrays.insert(0, program);
    Machine {
        count: 0,
        registers: [0; 8],
        arrays: arrays,
        array_list: Vec::new(),
        execution_finger: 0,
        halted: false
    }
}
