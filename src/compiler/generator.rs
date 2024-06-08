
use std::collections::HashMap;

use nova_tw::language::{Expression, ExpressionVisitor, Object, Statement, StatementVisitor, TokenType};

use crate::{bytecode::OpCode, instruction::{Instruction, InstructionBuilder}, object::{NovaFunction, NovaObject}, program::Program};

pub struct BytecodeGenerator {
    program: Program,
    error: Option<String>,
    temp_stack: Vec<()>,
    _frame_stack: Vec<()>,
    global_variables: HashMap<String, u32>,
    local_variable_count: u32,
    local_variable_indices: Vec<HashMap<String, u32>>,
    scope: u32,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            program: Program::default(),
            error: None,
            temp_stack: Vec::new(),
            _frame_stack: Vec::new(),
            global_variables: HashMap::new(),
            local_variable_count: 0,
            local_variable_indices: Vec::new(),
            scope: 0,
        }
    }

    pub fn generate_bytecode(mut self, statements: &Vec<Statement>) -> Result<Program, String> {
        for statement in statements {
            self.execute(statement);
            if let Some(error) = self.error {
                return Err(error);
            }
        }

        self.program.instructions.push(InstructionBuilder::new_halt_instruction());

        Ok(self.program)
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

    fn get_immutable_index(&mut self, immutable: &NovaObject) -> Instruction {
        if self.program.immutables.contains(immutable) {
            self.program.immutables.iter().position(|value| value == immutable).unwrap() as Instruction
        } else {
            self.program.immutables.push(immutable.clone());
            self.program.immutables.len() as Instruction - 1
        }
    }

    fn allocate_local(&mut self, name: &str) -> Instruction {
        let index = self.local_variable_count;
        self.local_variable_count += 1;

        let map = self.local_variable_indices.last_mut();
        if map.is_none() {
            self.generate_error("Error allocating local variable".to_string());
            return 0;
        }

        let map = map.unwrap();
        map.insert(name.to_string(), index);

        index
    }

    fn get_local_index(&mut self, name: &str) -> Option<Instruction> { 
        let mut scope = self.local_variable_indices.len() as isize - 1;

        while scope >= 0 {
            let map = self.local_variable_indices.get(scope as usize).unwrap();
            if let Some(&value)  = map.get(name) {
                return Some(value);
            }

            scope -= 1;
        }
        
        None
    }

    /// add an instruction to the program and return it's index
    fn add_instruction(&mut self, instruction: Instruction) -> Instruction {
        let index = self.program.instructions.len();
        self.program.instructions.push(instruction);
        index as Instruction
    }
}

impl ExpressionVisitor for BytecodeGenerator {
    type Output = ();

    fn visit_binary(&mut self, binary: &nova_tw::language::binary::Binary) -> Self::Output {

        self.evaluate(&binary.left);
        self.evaluate(&binary.right);

        let mut invert_condition = false;
        
        let opcode = match binary.operator.token_type {
            TokenType::Plus => OpCode::Add,
            TokenType::Minus => OpCode::Sub,
            TokenType::Slash => OpCode::Div,
            TokenType::Star => OpCode::Mul,
            TokenType::Caret => OpCode::Pow,
            TokenType::Percent => OpCode::Mod,
            TokenType::Less => OpCode::Less,
            TokenType::LessEqual => OpCode::LessEqual,
            TokenType::Greater => {
                invert_condition = true;
                OpCode::LessEqual
            }
            TokenType::GreaterEqual => {
                invert_condition = true;
                OpCode::Less
            },
            TokenType::EqualEqual => OpCode::Equal,
            TokenType::NotEqual => {
                invert_condition = true;
                OpCode::Equal
            }
            
            
            _ => {
                self.generate_error(format!("[Unhandled binary operator: {:?}]", binary.operator.token_type));
                return;
            }
        };

        let right_index = self.temp_stack.len() as Instruction - 1;
        let left_index = self.temp_stack.len() as Instruction - 2;

        self.temp_stack.pop();

        self.program.instructions.push(InstructionBuilder::new_binary_op_instruction(opcode, left_index, left_index, right_index));

        if invert_condition {
            self.add_instruction(InstructionBuilder::new_not_instruction(left_index));
        }

    }

    fn visit_unary(&mut self, unary: &nova_tw::language::unary::Unary) -> Self::Output {
        self.evaluate(&unary.right);

        let index = self.temp_stack.len() as Instruction - 1;
        match unary.operator.token_type {
            TokenType::Minus => {
                self.program.instructions.push(InstructionBuilder::new().add_opcode(OpCode::Neg).add_source_register_1(index).build())
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
                let immutable_index = self.get_immutable_index(&object);
                self.program.instructions.push(InstructionBuilder::new_load_constant_instruction(register_index, immutable_index))
            },

            Object::Callable(_) => todo!(),
            Object::Instance(_) => todo!(),
        }

        self.temp_stack.push(())
    }

    fn visit_call(&mut self, function: &nova_tw::language::call::Call) -> Self::Output {
        if let Expression::Variable(variable) = &function.callee {
            let name = variable.name.object.to_string();
            if name == "print" {
                for argument in &function.arguments {
                    self.evaluate(argument);
                    let source = self.temp_stack.len() as Instruction - 1;
                    self.temp_stack.pop();
                    self.program.instructions.push(InstructionBuilder::new_print_instruction(source, false));
                }
                return;
            }

            if name == "println" {
                for (index, argument) in function.arguments.iter().enumerate() {
                    self.evaluate(argument);
                    let source = self.temp_stack.len() as Instruction - 1;
                    self.temp_stack.pop();
                    let newline = if index == function.arguments.len() - 1 { true } else { false };
                    self.program.instructions.push(InstructionBuilder::new_print_instruction(source, newline));
                }
                return;
            }

            for argument in &function.arguments {
                self.evaluate(argument);
            }
    
            let parameters = function.arguments.len() as Instruction;
    
            let name_index = self.get_immutable_index(&NovaObject::String(Box::new(name)));
            self.program.instructions.push(InstructionBuilder::new_call_indirect_instruction(parameters, name_index));
            return;
        }

        
        self.generate_error(format!("Error compiling function call"));
    }

    fn visit_variable(&mut self, variable: &nova_tw::language::variable::Variable) -> Self::Output {
        let name = variable.name.object.to_string();
        if let Some(index) = self.get_local_index(name.as_str()) {
            let destination = self.temp_stack.len() as Instruction;
            self.program.instructions.push(InstructionBuilder::new_load_local(destination, index));
            self.temp_stack.push(());
            return;
        }
        
        let name = NovaObject::String(Box::new(name));
        let name_index = self.get_immutable_index(&name);
        let destination = self.temp_stack.len() as Instruction;
        self.program.instructions.push(InstructionBuilder::new_load_global_indirect(destination, name_index));
        self.temp_stack.push(());
        return;
    }

    fn visit_assign(&mut self, assign: &nova_tw::language::assignment::Assign) -> Self::Output {
        self.evaluate(&assign.value);
        let name = assign.name.object.to_string();

        if let Some(index) = self.get_local_index(name.as_str()) { // check if variable is a local
            let source = self.temp_stack.len() as Instruction - 1;
            self.temp_stack.pop();
            self.program.instructions.push(InstructionBuilder::new_store_local(source, index));
            return;
        }
        

        let name = NovaObject::String(Box::new(name));
        let name_index = self.get_immutable_index(&name);
        let source = self.temp_stack.len() as Instruction - 1;
        self.temp_stack.pop();
        self.program.instructions.push(InstructionBuilder::new_store_global_indirect(source, name_index));
    }

    fn visit_get(&mut self, _get: &nova_tw::language::assignment::Get) -> Self::Output {
        todo!()
    }

    fn visit_set(&mut self, _set: &nova_tw::language::assignment::Set) -> Self::Output {
        todo!()
    }
}

impl StatementVisitor for BytecodeGenerator {
    type Output = ();

    fn visit_none(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_if(&mut self, if_statement: &nova_tw::language::IfStatement) -> Self::Output {
        self.evaluate(&if_statement.condition);

        let source = self.temp_stack.len() as Instruction - 1;
        self.temp_stack.pop();

        self.add_instruction(InstructionBuilder::new_jump_false_instruction(source));
        let jump_then_branch = self.add_instruction(InstructionBuilder::new_jump_instruction(1, true));
        self.execute(&if_statement.then_branch);
        let current = self.program.instructions.len() as Instruction - 1;
        let offset = current - jump_then_branch;
        self.program.instructions[jump_then_branch as usize] = InstructionBuilder::new_jump_instruction(offset+2, true);
        
        if let Some(else_branch) = &if_statement.else_branch {
            let jump_else_branch = self.add_instruction(InstructionBuilder::new_jump_instruction(1, true));
            self.execute(else_branch);
            let current = self.program.instructions.len() as Instruction - 1;
            let offset = current - jump_else_branch;
            self.program.instructions[jump_else_branch as usize] = InstructionBuilder::new_jump_instruction(offset+1, true);
        }

    }

    fn visit_while(&mut self, while_loop: &nova_tw::language::WhileLoop) -> Self::Output {
        let loop_start = self.program.instructions.len() as Instruction;
        self.evaluate(&while_loop.condition);
        let source = self.temp_stack.len() as Instruction - 1;
        self.temp_stack.pop();

        self.add_instruction(InstructionBuilder::new_jump_false_instruction(source));
        let jump_loop_index = self.add_instruction(InstructionBuilder::new_jump_instruction(1, true));

        self.execute(&while_loop.body);

        let current_index = self.program.instructions.len() as Instruction - 1;
        let back_offset = current_index - loop_start;

        self.add_instruction(InstructionBuilder::new_jump_instruction(back_offset, false));
        let current_index = self.program.instructions.len() as Instruction - 1;
        let jump_forward_offset = current_index - jump_loop_index;
        self.program.instructions[jump_loop_index as usize] = InstructionBuilder::new_jump_instruction(jump_forward_offset+1, true);
    }

    fn visit_block(&mut self, block: &nova_tw::language::Block) -> Self::Output {
        self.scope += 1;
        self.local_variable_indices.push(HashMap::new());
        self.program.instructions.push(InstructionBuilder::new_allocate_local(1)); // placeholder instruction
        let placeholder_index = self.program.instructions.len() as Instruction - 1;

        for statement in &block.statements {
            self.execute(statement);
        }

        let indices =self.local_variable_indices.pop().unwrap();
        let num_locals = indices.len();
        self.program.instructions[placeholder_index  as usize] = InstructionBuilder::new_allocate_local(num_locals as Instruction);
        self.program.instructions.push(InstructionBuilder::new_deallocate_local(num_locals as Instruction));

        self.scope -= 1;
        self.local_variable_count -= num_locals as u32;
    }

    fn visit_function_statement(&mut self, function_statement: &nova_tw::language::function::FunctionStatement) -> Self::Output {
        let jump_index = self.add_instruction(0 as Instruction); // placeholder instruction
        self.scope += 1;
        self.local_variable_indices.push(HashMap::new());

        let current_instruction_index = self.program.instructions.len() as Instruction; 
        let function_immutable = NovaObject::NovaFunction(
            NovaFunction{
                name: Box::new(function_statement.name.object.to_string()),
                address: current_instruction_index,
                arity: function_statement.parameters.len() as Instruction,
                is_method: false
            }
        );

        let string_immutable = NovaObject::String(
            Box::new(function_statement.name.object.to_string())
        );

        let _ = self.get_immutable_index(&string_immutable);
        let _ = self.get_immutable_index(&function_immutable);
        
        //self.add_instruction(InstructionBuilder::new_call_indirect_instruction(number_of_parameters, function_name_index));
        let mut parameter_locals = Vec::new();

        // loop through the parameter list and allocate local variables
        for parameter in &function_statement.parameters {
            let index = self.allocate_local(parameter.object.to_string().as_str());
            parameter_locals.push(index);
        }

        let place_holder = self.add_instruction(InstructionBuilder::new_allocate_local(1 as Instruction));
        
        for (register_index, &local_index) in parameter_locals.iter().enumerate() {
            self.add_instruction(InstructionBuilder::new_store_local(register_index as Instruction, local_index));
        }

        for statement in function_statement.body.statements.iter() {
            self.execute(statement);
        }

        let indices =self.local_variable_indices.pop().unwrap();
        let num_locals = indices.len();

        self.program.instructions[place_holder as usize] = InstructionBuilder::new_allocate_local(num_locals as Instruction);
        self.add_instruction(InstructionBuilder::new_deallocate_local(num_locals as Instruction));
        self.add_instruction(InstructionBuilder::new_return_none_instruction());
        self.scope -= 1;
        self.local_variable_count -= num_locals as u32;

        let current = self.program.instructions.len() as Instruction;
        self.program.instructions[jump_index as usize] = InstructionBuilder::new_jump_instruction(current - jump_index, true);
    }

    fn visit_return(&mut self, _return_statement: &Option<nova_tw::language::Expression>) -> Self::Output {
        todo!()
    }

    fn visit_var_declaration(&mut self, var_declaration: &nova_tw::language::declaration::VariableDeclaration) -> Self::Output {
        let mut initialized = false;
        if let Some(initializer) = &var_declaration.initializer {
            self.evaluate(initializer);
            initialized = true;
        }

        let name_str = var_declaration.name.object.to_string();
        if self.scope == 0 {// global scope
           
            let name = NovaObject::String(Box::new(name_str.clone()));
            
            let name_index = self.get_immutable_index(&name);
            self.program.instructions.push(InstructionBuilder::new_define_global_indirect(name_index));
            self.global_variables.insert(name_str, name_index);

            if initialized {
                let source = self.temp_stack.len() as Instruction - 1;
                self.temp_stack.pop();
                self.program.instructions.push(InstructionBuilder::new_store_global_indirect(source, name_index));
            }
            return;
        }

        let index = self.allocate_local(name_str.as_str());
        if initialized {
            let source = self.temp_stack.len() as Instruction - 1;
            self.temp_stack.pop();
            self.program.instructions.push(InstructionBuilder::new_store_local(source, index));
        }
    }

    fn visit_expression_statement(&mut self, expression_statement: &nova_tw::language::Expression) -> Self::Output {
        self.evaluate(expression_statement);
    }

    fn visit_class_statement(&mut self, _class_statement: &nova_tw::language::class::ClassStatement) -> Self::Output {
        todo!()
    }

    fn visit_include(&mut self, _include: &nova_tw::language::Include) -> Self::Output {
        todo!()
    }
}