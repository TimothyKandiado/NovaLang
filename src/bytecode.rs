#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    /// Copy a value between registers(MOVE A <- B)
    Move,
    /// Load a constant into a register (LOAD A <- K)
    LoadK, 
    /// Load nil values into a range of registers
    LoadNil,
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
    LT,
    /// Less than or equal Test (Jump if false) (A < B) ? Skip Jump : Jump
    LE,
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
    /// Return function value
    Return,

    Halt,
}
