#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    NoInstruction,
    /// Copy a value between registers(MOVE A <- B)
    Move,
    /// Load a constant into a register (LOAD A <- K)
    LoadK,
    /// Load nil values into a range of registers
    LoadNil,
    /// Load Boolean values into a register
    LoadBool,
    /// Load int32 values into a register
    LoadInt32,
    /// Load int64 values into a register
    LoadInt64,
    /// Load float32 values into a register
    LoadFloat32,
    /// Load float64 values into the register
    LoadFloat64,
    /// Load a return value (LOADRETURN DR SR1)
    LoadReturn,
    /// Clear return value in register
    ClearReturn,
    /// Prepare an object method for calling
    This,
    /// Addition operator
    Add,
    /// Subtraction operator
    Sub,
    /// Multiplication operator
    Mul,
    /// Division operator
    Div,
    /// Modulus (remainder) operator
    Mod,
    /// Exponentation operator
    Pow,
    /// Unary Minus
    Neg,
    /// Logical Not
    Not,
    /// Logical And
    And,
    /// Logical Or
    Or,
    /// Less than Test (Jump if false) (A < B) ? Skip Jump : Jump
    Less,
    /// Less than or equal Test (Jump if false) (A < B) ? Skip Jump : Jump
    LessEqual,
    /// Equality test
    Equal,
    /// Jump if a condition returned false
    JumpFalse,
    /// Unconditional Jump with Offset
    Jump,
    /// Define Global Variable by looking up variable name
    DefineGlobalIndirect,
    /// Store value in global variable by looking up variable name
    StoreGlobalIndirect,
    /// Load value from Global Variable looking up variable name
    LoadGlobalIndirect,
    /// Load Global Variable
    LoadGlobal,
    /// Allocate space for local variables
    AllocateLocal,
    /// Deallocate space for local variables
    DeallocateLocal,
    /// Store local variable
    StoreLocal,
    /// Load local variable
    LoadLocal,
    /// Print value in register
    Print,
    /// Invoke call
    Invoke,
    /// While loop
    While,
    /// Unconditional Loop
    Loop,
    /// Exit loop
    Break,
    /// Push new stack frame
    NewFrame,
    /// Return null
    ReturnNone,
    /// Return value
    ReturnVal,
    /// Stop the interpreter
    Halt,
}

pub const BYTECODE_COUNT: u32 = 44;

pub const BYTECODE_LOOKUP_TABLE: [OpCode; 44] = [
    OpCode::NoInstruction,
    OpCode::Move,
    OpCode::LoadK,
    OpCode::LoadNil,
    OpCode::LoadBool,
    OpCode::LoadInt32,
    OpCode::LoadInt64,
    OpCode::LoadFloat32,
    OpCode::LoadFloat64,
    OpCode::LoadReturn,
    OpCode::ClearReturn,
    OpCode::This,
    OpCode::Add,
    OpCode::Sub,
    OpCode::Mul,
    OpCode::Div,
    OpCode::Mod,
    OpCode::Pow,
    OpCode::Neg,
    OpCode::Not,
    OpCode::And,
    OpCode::Or,
    OpCode::Less,
    OpCode::LessEqual,
    OpCode::Equal,
    OpCode::JumpFalse,
    OpCode::Jump,
    OpCode::DefineGlobalIndirect,
    OpCode::StoreGlobalIndirect,
    OpCode::LoadGlobalIndirect,
    OpCode::LoadGlobal,
    OpCode::AllocateLocal,
    OpCode::DeallocateLocal,
    OpCode::StoreLocal,
    OpCode::LoadLocal,
    OpCode::Print,
    OpCode::Invoke,
    OpCode::While,
    OpCode::Loop,
    OpCode::Break,
    OpCode::NewFrame,
    OpCode::ReturnNone,
    OpCode::ReturnVal,
    OpCode::Halt,
];

impl OpCode {
    #[inline(always)]
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}
