#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    /// Copy a value between registers(MOVE A <- B)
    Move,
    /// Load a constant into a register (LOAD A <- K)
    LoadK,
    /// Load nil values into a range of registers
    LoadNil,
    /// Load Boolean values into a register
    LoadBool,
    /// Load float values into a register
    LoadFloat,
    /// Store variable in memory
    StoreV,
    /// Load variable into a register
    LoadV,
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
    LessJump,
    /// Less than or equal Test (Jump if false) (A < B) ? Skip Jump : Jump
    LessEqualJump,
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
    /// Call a function
    Call,
    /// While loop
    While,
    /// Unconditional Loop
    Loop,
    /// Exit loop
    Break,
    /// Push new stack frame
    NewFrame,
    /// Return function value
    Return,

    Halt,
}

impl OpCode {
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}