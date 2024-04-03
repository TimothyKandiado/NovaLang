use std::error;

use nova_tw::language::{Expression, ExpressionVisitor, Object, Statement, StatementVisitor, TokenType};

use crate::{bytecode::OpCode, instruction::{self, Instruction, InstructionBuilder}, object::NovaObject, program::Program};

pub struct BytecodeGenerator {
    program: Program,
    error: Option<String>,
    temp_stack: Vec<()>
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            program: Program::default(),
            error: None,
            temp_stack: Vec::new()
        }
    }

    pub fn generate_bytecode(mut self, statements: &Vec<Statement>) -> Program {
        for statement in statements {
            self.execute(statement);
            if self.error.is_some() {
                
                break;
            }
        }

        self.program.instructions.push(InstructionBuilder::new_halt_instruction());

        self.program
    }

    fn execute(&mut self, statement: &Statement) {
        statement.accept(self);
    }

    fn evaluate(&mut self, expression: &Expression) {
        expression.accept(self)
    }

    fn generate_error(&mut self, error: String) {
        if self.error.is_some() {
            return;
        }

        self.error = Some(format!("[Bytecode Gen Error]: {}",error))
    }
}

impl ExpressionVisitor for BytecodeGenerator {
    type Output = ();

    fn visit_binary(&mut self, binary: &nova_tw::language::binary::Binary) -> Self::Output {

        self.evaluate(&binary.left);
        self.evaluate(&binary.right);
        
        let opcode = match binary.operator.token_type {
            TokenType::Plus => OpCode::Add,
            TokenType::Minus => OpCode::Sub,
            TokenType::Slash => OpCode::Div,
            TokenType::Star => OpCode::Mul,
            TokenType::Caret => OpCode::Pow,
            
            _ => {
                self.generate_error(format!("[Unhandled binary operator: {:?}]", binary.operator.token_type));
                return;
            }
        };

        let right_index = self.temp_stack.len() as Instruction - 1;
        let left_index = self.temp_stack.len() as Instruction - 2;

        self.temp_stack.pop();

        self.program.instructions.push(InstructionBuilder::new_binary_op_instruction(opcode, left_index, left_index, right_index))

    }

    fn visit_unary(&mut self, unary: &nova_tw::language::unary::Unary) -> Self::Output {
        self.evaluate(&unary.right);

        let index = self.temp_stack.len() as Instruction - 1;
        match unary.operator.token_type {
            TokenType::Minus => {
                self.program.instructions.push(InstructionBuilder::new().add_opcode(OpCode::Neg).add_destination_register(index).add_source_register_1(index).build())
            }

            _ => {
                self.generate_error(format!("[Unhandled unary operator: {:?}]", unary.operator.token_type));
                return;
            }
        }
    }

    fn visit_grouping(&mut self, grouping: &nova_tw::language::grouping::Grouping) -> Self::Output {
        self.evaluate(&grouping.expression)
    }

    fn visit_literal(&mut self, literal: &nova_tw::language::literal::Literal) -> Self::Output {
        let object = literal.object.clone();
        let register_index = self.temp_stack.len() as Instruction;
        match object {
            Object::Number(number) => {
                let number = number as f32;
                self.program.instructions.push(
                    InstructionBuilder::new_load_float32_instruction(register_index)
                );
                self.program.instructions.push(number.to_bits());
            },

            Object::Bool(bool) => {
                self.program.instructions.push(InstructionBuilder::new_load_bool(register_index, bool as Instruction))
            }

            Object::None => self.program.instructions.push(InstructionBuilder::new().add_opcode(OpCode::LoadNil).add_destination_register(register_index).build()),
            Object::String(string) => {
                let object = NovaObject::String(Box::new(string));
                let immutable_index = if self.program.immutables.contains(&object) {
                    self.program.immutables.iter().position(|value| value == &object).unwrap()
                } else {
                    self.program.immutables.push(object);
                    self.program.immutables.len() - 1
                } as Instruction;
                self.program.instructions.push(InstructionBuilder::new_load_constant_instruction(register_index, immutable_index))
            },

            Object::Callable(_) => todo!(),
            Object::Instance(_) => todo!(),
        }

        self.temp_stack.push(())
    }

    fn visit_call(&mut self, math_function: &nova_tw::language::call::Call) -> Self::Output {
        todo!()
    }

    fn visit_variable(&mut self, variable: &nova_tw::language::variable::Variable) -> Self::Output {
        todo!()
    }

    fn visit_assign(&mut self, assign: &nova_tw::language::assignment::Assign) -> Self::Output {
        todo!()
    }

    fn visit_get(&mut self, get: &nova_tw::language::assignment::Get) -> Self::Output {
        todo!()
    }

    fn visit_set(&mut self, set: &nova_tw::language::assignment::Set) -> Self::Output {
        todo!()
    }
}

impl StatementVisitor for BytecodeGenerator {
    type Output = ();

    fn visit_none(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_if(&mut self, if_statement: &nova_tw::language::IfStatement) -> Self::Output {
        todo!()
    }

    fn visit_while(&mut self, while_loop: &nova_tw::language::WhileLoop) -> Self::Output {
        todo!()
    }

    fn visit_block(&mut self, block: &nova_tw::language::Block) -> Self::Output {
        todo!()
    }

    fn visit_function_statement(&mut self, function_statement: &nova_tw::language::function::FunctionStatement) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self, return_statement: &Option<nova_tw::language::Expression>) -> Self::Output {
        todo!()
    }

    fn visit_var_declaration(&mut self, var_declaration: &nova_tw::language::declaration::VariableDeclaration) -> Self::Output {
        todo!()
    }

    fn visit_expression_statement(&mut self, expression_statement: &nova_tw::language::Expression) -> Self::Output {
        self.evaluate(expression_statement);
    }

    fn visit_class_statement(&mut self, class_statement: &nova_tw::language::class::ClassStatement) -> Self::Output {
        todo!()
    }

    fn visit_include(&mut self, include: &nova_tw::language::Include) -> Self::Output {
        todo!()
    }
}