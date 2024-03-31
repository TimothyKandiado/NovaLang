# Bytecode
All instructions are unsigned 32bit integers
First 6 bits are reserved for OpCodes

Key
- DR: destination register (next 4 bits)
- SR1: source register 1 (next 4 bits)
- SR2: source register 2 (last 4 bits)
- IMM: immutable value index (last 16 bits)
- SW: boolean switch (1 - true, 0 - false)

## MOVE DR SR1
Copy contents from SR1 to DR1

## LOADK DR IMM
Look Up index IMM in constant(immutable table/array) and place its address in Register

## LOADNIL DR
Set DR to Nil/None

## LOADFLOAT DR
Load a float into destination register.
The float occupies the next Instruction.

## ADD DR SR1 SR2
Perform addition on contents of SR1 and SR2 and place result in DR
SUB, MUL, DIV, POW and MOD work the same way with respective operation

## LESSJUMP SR1 SR2
Check if SR1 less than SR2
If true skip the next Jump Instruction.
If false, consumes next Jump instruction and performs It 
LESSEQUALJUMP Works similarly

## JUMP SW IMM
Jump by offsetting the program counter by value IMM
If SW is true perform a backward Jump

## Print SR1
Print content of SR1 to stdout

## Halt
Stop program execution

## Call IMM
Perform a operation

## NewFrame
Creates a new call frame
Followed by a jump to the function

## Return
Exit a function
Pops the most recent call frame, quits programs if there are no more
Should restore the registry values to before they call was performed