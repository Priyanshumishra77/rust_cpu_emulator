use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use Operand::Memory;
use crate::cpu::{CPSR, GENERAL_ARG_REG_CNT, SP};
use crate::cpu::LR;
use crate::cpu::PC;
use crate::cpu::FP;
use crate::instructions::instructions::Operand::{Code, Immediate, Register, Unused};

#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Opcode {
    ADD,
    SUB,
    MUL,
    SDIV,
    ADR,
    LDR,
    STR,
    NOP,
    PRINTR,
    MOV,
    B,
    BX,
    BL,
    CBZ,
    CBNZ,
    // Acts like a poison pill. It isn't a public instruction.
    EXIT,
    NEG,
    AND,
    ORR,
    EOR,
    MVN,
    CMP,
    BEQ,
    BNE,
    BLE,
    BLT,
    BGE,
    BGT,
}

pub(crate) fn mnemonic(opcode: Opcode) -> &'static str {
    match opcode {
        Opcode::ADD => "ADD",
        Opcode::SUB => "SUB",
        Opcode::MUL => "MUL",
        Opcode::SDIV => "SDIV",
        Opcode::NEG => "NEG",
        Opcode::ADR => "ADR",
        Opcode::LDR => "LDR",
        Opcode::STR => "STR",
        Opcode::NOP => "NOP",
        Opcode::PRINTR => "PRINTR",
        Opcode::MOV => "PRINTR",
        Opcode::B => "B",
        Opcode::BX => "BX",
        Opcode::BL => "BL",
        Opcode::CBZ => "CBZ",
        Opcode::CBNZ => "CBNZ",
        Opcode::AND => "AND",
        Opcode::ORR => "ORR",
        Opcode::EOR => "EOR",
        Opcode::MVN => "MVN",
        Opcode::EXIT => "EXIT",
        Opcode::CMP => "CMP",
        Opcode::BEQ => "BEQ",
        Opcode::BNE => "BNE",
        Opcode::BLE => "BLE",
        Opcode::BLT => "BLT",
        Opcode::BGE => "BGE",
        Opcode::BGT => "BGT",
    }
}

pub(crate) fn get_opcode(mnemonic: &str) -> Option<Opcode> {
    let string = mnemonic.to_uppercase();
    let mnemonic_uppercased = string.as_str();

    match mnemonic_uppercased {
        "ADD" => Some(Opcode::ADD),
        "SUB" => Some(Opcode::SUB),
        "MUL" => Some(Opcode::MUL),
        "SDIV" => Some(Opcode::SDIV),
        "NEG" => Some(Opcode::NEG),
        "ADR" => Some(Opcode::ADR),
        "LDR" => Some(Opcode::LDR),
        "STR" => Some(Opcode::STR),
        "NOP" => Some(Opcode::NOP),
        "PRINTR" => Some(Opcode::PRINTR),
        "MOV" => Some(Opcode::MOV),
        "B" => Some(Opcode::B),
        "BX" => Some(Opcode::BX),
        "CBZ" => Some(Opcode::CBZ),
        "CBNZ" => Some(Opcode::CBNZ),
        "AND" => Some(Opcode::AND),
        "ORR" => Some(Opcode::ORR),
        "EOR" => Some(Opcode::EOR),
        "MVN" => Some(Opcode::MVN),
        "BL" => Some(Opcode::BL),
        "EXIT" => Some(Opcode::EXIT),
        "CMP" => Some(Opcode::CMP),
        "BEQ" => Some(Opcode::BEQ),
        "BNE" => Some(Opcode::BNE),
        "BLE" => Some(Opcode::BLE),
        "BLT" => Some(Opcode::BLT),
        "BGE" => Some(Opcode::BGE),
        "BGT" => Some(Opcode::BGT),
        _ => None,
    }
}

pub(crate) fn get_register(name: &str) -> Option<u16> {
    let name_uppercased = name.to_uppercase();

    match name_uppercased.as_str() {
        "SP" => Some(SP),
        "LR" => Some(LR),
        "PC" => Some(PC),
        "FP" => Some(FP),
        _ => {
            let reg_name = &name_uppercased[1..];
            let reg: u16 = reg_name.parse().unwrap();

            if reg >= GENERAL_ARG_REG_CNT {
                return None;
            }
            Some(reg)
        }
    }
}

pub(crate) fn create_instr(opcode: Opcode,
                           operands: &Vec<Operand>,
                           loc: SourceLocation) -> Result<Instr, String> {
    let mut instr = Instr {
        cycles: 1,
        opcode,
        source_cnt: 0,
        source: [Unused, Unused, Unused],
        sink_cnt: 0,
        sink: [Unused, Unused],
        loc: Some(loc),
        mem_stores: 0,
        is_control: false,
    };

    match opcode {
        Opcode::SUB |
        Opcode::MUL |
        Opcode::SDIV |
        Opcode::AND |
        Opcode::ORR |
        Opcode::EOR |
        Opcode::ADD => {
            validate_operand_count(3, operands, opcode, loc)?;

            instr.sink_cnt = 1;
            instr.sink[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.source_cnt = 2;
            instr.source[0] = validate_operand(1, operands, opcode, &[Register(0)])?;
            instr.source[1] = validate_operand(2, operands, opcode, &[Register(0), Immediate(0)])?;
        }
        Opcode::ADR => { panic!() }
        Opcode::LDR => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.sink_cnt = 1;
            instr.sink[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(1, operands, opcode, &[Register(0)])?
        }
        Opcode::STR => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.mem_stores = 1;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.sink_cnt = 1;
            instr.sink[0] = validate_operand(1, operands, opcode, &[Register(0)])?;
        }
        Opcode::NOP => {
            validate_operand_count(0, operands, opcode, loc)?;
        }
        Opcode::PRINTR => {
            validate_operand_count(1, operands, opcode, loc)?;

            instr.sink_cnt = 0;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(0, operands, opcode, &[Register(0)])?;
        }
        Opcode::MOV => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.sink_cnt = 1;
            instr.sink[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(1, operands, opcode, &[Immediate(0), Register(0)])?
        }
        Opcode::B => {
            validate_operand_count(1, operands, opcode, loc)?;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(0, operands, opcode, &[Code(0)])?;

            instr.sink_cnt = 1;
            instr.sink[0] = Register(PC);
        }
        Opcode::BX => {
            validate_operand_count(1, operands, opcode, loc)?;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.sink_cnt = 1;
            instr.sink[0] = Register(PC);
        }
        Opcode::BL => {
            validate_operand_count(1, operands, opcode, loc)?;

            instr.source_cnt = 2;
            instr.source[0] = validate_operand(0, operands, opcode, &[Code(0)])?;
            instr.source[1] = Register(PC);

            instr.sink_cnt = 2;
            instr.sink[0] = Register(LR);
            instr.sink[1] = Register(PC);
        }
        Opcode::CBZ |
        Opcode::CBNZ => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.source_cnt = 3;
            instr.source[0] = validate_operand(0, operands, opcode, &[Register(0)])?;
            instr.source[1] = validate_operand(1, operands, opcode, &[Code(0)])?;
            instr.source[2] = Register(PC);

            instr.sink_cnt = 1;
            instr.sink[0] = Register(PC);
        }
        Opcode::EXIT => {
            validate_operand_count(0, operands, opcode, loc)?;

            instr.is_control = true;
        }
        Opcode::NEG => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.sink_cnt = 1;
            instr.sink[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(1, operands, opcode, &[Register(0)])?;
        }
        Opcode::MVN => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.sink_cnt = 1;
            instr.sink[0] = validate_operand(0, operands, opcode, &[Register(0)])?;

            instr.source_cnt = 1;
            instr.source[0] = validate_operand(1, operands, opcode, &[Immediate(0), Register(0)])?;
        }
        Opcode::CMP => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.source_cnt = 3;
            instr.source[0] = validate_operand(0, operands, opcode, &[Register(0)])?;
            instr.source[1] = validate_operand(1, operands, opcode, &[Immediate(0), Register(0)])?;
            instr.source[2] = Register(CPSR);

            instr.sink_cnt = 1;
            instr.sink[0] = Register(CPSR);
        }
        Opcode::BEQ | Opcode::BNE | Opcode::BLT | Opcode::BLE | Opcode::BGT | Opcode::BGE => {
            validate_operand_count(2, operands, opcode, loc)?;

            instr.source_cnt = 3;
            instr.source[0] = validate_operand(0, operands, opcode, &[Code(0)])?;
            instr.source[1] = Register(CPSR);
            instr.source[2] = Register(PC);

            instr.sink_cnt = 1;
            instr.sink[0] = Register(PC);
        }
    }

    instr.is_control = is_control(&instr);
    return Ok(instr);
}

fn validate_operand_count(expected: usize, operands: &Vec<Operand>, opcode: Opcode, loc: SourceLocation) -> Result<(), String> {
    if operands.len() != expected {
        return Err(format!("Operand count mismatch. {:?} expects {} arguments, but {} are provided at {}:{}",
                           opcode, expected, operands.len(), loc.line, loc.column));
    }
    Ok(())
}

fn validate_operand(op_index: usize, operands: &Vec<Operand>, opcode: Opcode, acceptable_types: &[Operand]) -> Result<Operand, String> {
    let operand = operands[op_index];

    for &typ in acceptable_types {
        if std::mem::discriminant(&operand) == std::mem::discriminant(&typ) {
            return Ok(operand);
        }
    }
    let acceptable_names: Vec<&str> = acceptable_types.iter().map(|t| t.base_name()).collect();
    let acceptable_names_str = acceptable_names.join(", ");

    Err(format!("Operand type mismatch. {:?} expects {} as argument nr {}, but {} was provided",
                opcode, acceptable_names_str, op_index + 1, operand.base_name()))
}

fn is_control(instr: &Instr) -> bool {
    instr.source.iter().any(|op| is_control_operand(op)) ||
        instr.sink.iter().any(|op| is_control_operand(op))
}

fn is_control_operand(op: &Operand) -> bool {
    matches!(op, Register(register) if *register == PC)
}

pub(crate) const NOP: Instr = Instr {
    cycles: 1,
    opcode: Opcode::NOP,
    source_cnt: 0,
    source: [Operand::Unused, Operand::Unused, Operand::Unused],
    sink_cnt: 0,
    sink: [Operand::Unused, Operand::Unused],
    loc: None,
    mem_stores: 0,
    is_control: false,
};

pub(crate) const EXIT: Instr = Instr {
    cycles: 1,
    opcode: Opcode::EXIT,
    source_cnt: 0,
    source: [Operand::Unused, Operand::Unused, Operand::Unused],
    sink_cnt: 0,
    sink: [Operand::Unused, Operand::Unused],
    loc: None,
    mem_stores: 0,
    is_control: false,
};

pub(crate) type RegisterType = u16;
pub(crate) type WordType = i64;

// The InstrQueue sits between frontend and backend
pub(crate) struct InstrQueue {
    capacity: u16,
    head: u64,
    tail: u64,
    instructions: Vec<Rc<Instr>>,
}

impl InstrQueue {
    pub fn new(capacity: u16) -> Self {
        let mut instructions = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            instructions.push(Rc::new(NOP));
        }

        InstrQueue {
            capacity,
            head: 0,
            tail: 0,
            instructions,
        }
    }

    pub fn size(&self) -> u16 {
        (self.tail - self.head) as u16
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        self.size() == self.capacity
    }

    pub fn enqueue(&mut self, instr: Rc<Instr>) {
        assert!(!self.is_full(), "Can't enqueue when InstrQueue is empty.");

        let index = (self.tail % self.capacity as u64) as usize;
        self.instructions[index] = instr;
        self.tail += 1;
    }

    pub fn dequeue(&mut self) {
        assert!(!self.is_empty(), "Can't dequeue when InstrQueue is empty.");
        self.head += 1;
    }

    pub fn peek(&self) -> Rc<Instr> {
        assert!(!self.is_empty(), "Can't peek when InstrQueue is empty.");

        let index = (self.head % self.capacity as u64) as usize;
        return Rc::clone(&self.instructions[index]);
    }
}

// The maximum number of source (input) operands for an instruction.
pub(crate) const MAX_SOURCE_COUNT: u8 = 3;
pub(crate) const MAX_SINK_COUNT: u8 = 2;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Instr {
    pub(crate) cycles: u8,
    pub(crate) opcode: Opcode,
    pub(crate) source_cnt: u8,
    pub(crate) source: [Operand; MAX_SOURCE_COUNT as usize],
    pub(crate) sink_cnt: u8,
    pub(crate) sink: [Operand; MAX_SINK_COUNT as usize],
    pub(crate) loc: Option<SourceLocation>,
    pub(crate) mem_stores: u8,
    // True if the instruction is a control instruction; so a partly serializing instruction (no other instructions)
    pub(crate) is_control: bool,
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", mnemonic(self.opcode))?;

        match self.opcode {
            Opcode::ADD |
            Opcode::SUB |
            Opcode::MUL |
            Opcode::SDIV |
            Opcode::AND |
            Opcode::ORR |
            Opcode::EOR => write!(f, "{}, {}, {}", self.sink[0], self.source[0], self.source[1])?,
            Opcode::LDR => write!(f, "{}, {}", self.sink[0], self.source[0])?,
            Opcode::STR => write!(f, "{}, {}", self.source[0], self.sink[0])?,
            Opcode::MOV => write!(f, "{}, {}", self.sink[0], self.source[0])?,
            Opcode::NOP => {}
            Opcode::ADR => write!(f, "{}, {}", self.sink[0], self.source[0])?,
            Opcode::PRINTR => write!(f, "{}", self.source[0])?,
            Opcode::B |
            Opcode::BX |
            Opcode::BL => write!(f, "{}", self.source[0])?,
            Opcode::CBZ |
            Opcode::CBNZ => write!(f, "{}, {}", self.source[0], self.source[1])?,
            Opcode::NEG => write!(f, "{}, {}", self.sink[0], self.source[0])?,
            Opcode::MVN => write!(f, "{}, {}", self.sink[0], self.source[0])?,
            Opcode::CMP => write!(f, "{}, {}", self.source[0], self.source[1])?,
            Opcode::EXIT => {}
            Opcode::BEQ | Opcode::BNE | Opcode::BLT | Opcode::BLE | Opcode::BGT | Opcode::BGE =>
                write!(f, "{}", self.source[0])?,
        }

        if let Some(loc) = self.loc {
            write!(f, " ; {}:{}", loc.line, loc.column)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Operand {
    Register(RegisterType),
    // The operand is directly specified in the instruction itself.
    Immediate(WordType),

    // todo: rename to direct?
    Memory(WordType),

    Code(WordType),

    Unused,
}


impl Operand {
    pub fn base_name(&self) -> &str {
        match self {
            Register(_) => "Register",
            Immediate(_) => "Immediate",
            Memory(_) => "Memory",
            Code(_) => "Code",
            Unused => "Unused",
        }
    }
}


impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register(reg) => {
                match *reg as u16 {
                    FP => write!(f, "FP"),
                    LR => write!(f, "LR"),
                    SP => write!(f, "SP"),
                    PC => write!(f, "PC"),
                    CPSR => write!(f, "CPSR"),
                    _ => write!(f, "R{}", reg),
                }
            }  // Add a comma here
            Immediate(val) => write!(f, "{}", val),
            Memory(addr) => write!(f, "[{}]", addr),
            Code(addr) => write!(f, "[{}]", addr),
            Unused => write!(f, "Unused"),
        }
    }
}

//Indexed(u8, i16),   // Indexed addressing mode (base register and offset).
//Indirect(u8),

impl Operand {
    pub(crate) fn get_register(&self) -> RegisterType {
        match *self {
            Operand::Register(reg) => reg,
            _ => panic!("Operation is not a Register but of type {:?}", self),
        }
    }

    pub(crate) fn get_constant(&self) -> WordType {
        match self {
            Operand::Immediate(constant) => *constant,
            _ => panic!("Operand is not a Constant but of type {:?}", self),
        }
    }

    pub(crate) fn get_code_address(&self) -> WordType {
        match self {
            Operand::Code(constant) => *constant,
            _ => panic!("Operand is not a Code but of type {:?}", self),
        }
    }

    pub(crate) fn get_memory_addr(&self) -> WordType {
        match self {
            Memory(addr) => *addr,
            _ => panic!("Operand is not a Memory but of type {:?}", self),
        }
    }
}

pub(crate) struct Data {
    pub(crate) value: WordType,
    pub(crate) offset: u64,
}

pub(crate) struct Program {
    pub(crate) data_items: HashMap::<String, Rc<Data>>,
    pub(crate) code: Vec<Rc<Instr>>,
    pub(crate) entry_point: usize,
}

impl Program {
    pub fn get_instr(&self, pos: usize) -> Rc<Instr> {
        Rc::clone(&self.code[pos])
    }
}

