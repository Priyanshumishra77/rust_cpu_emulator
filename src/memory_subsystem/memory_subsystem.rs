use std::rc::Rc;

use crate::cpu::CPUConfig;
use crate::instructions::instructions::{Program, WordType};
use crate::memory_subsystem::store_buffer::StoreBuffer;

pub(crate) struct MemorySubsystem {
    pub(crate) memory: Vec<WordType>,
    pub(crate) sb: StoreBuffer,
}

impl MemorySubsystem {
    pub fn new(cpu_config: &CPUConfig) -> MemorySubsystem {
        let mut memory = Vec::with_capacity(cpu_config.memory_size as usize);

        for _ in 0..cpu_config.memory_size {
            memory.push(0);
        }

        let sb = StoreBuffer::new(cpu_config);

        MemorySubsystem {
            memory,
            sb,
        }
    }

    pub(crate) fn init(&mut self, program: &Rc<Program>) {
        for k in 0..self.memory.len() {
            self.memory[k] = 0;
        }

        for data in program.data_items.values() {
            self.memory[data.offset as usize] = data.value;
        }
    }

    pub fn do_cycle(&mut self) {
        self.sb.do_cycle(&mut self.memory);
    }
}



