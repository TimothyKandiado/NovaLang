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
    LESSJ,
    /// Less than or equal Test (Jump if false) (A < B) ? Skip Jump : Jump
    LESSEQUALJ,
    /// Unconditional Jump with Offset
    Jump,
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
