#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    /// Copy a value between registers
    Move,
    /// Load a constant into a register
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
    /// Length
    Len,
    /// Unconditional Jump
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
