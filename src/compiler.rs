use core::fmt;
use std::{fs::File, io::Write, path::Path, process::Command};

use crate::{
    lexer::BinaryOperator,
    parser::{Expression, Function, Local, LocalStack, Parser, Program, Scope, Statement},
};

#[derive(Clone)]
enum Register {
    R1(usize),
    R2(usize),
    R3(usize),
    R4(usize),
    R5(usize),
    R6(usize),
    R7(usize),
    R8(usize),
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::R1(size) => match size {
                8 => write!(f, "al"),
                16 => write!(f, "ax"),
                32 => write!(f, "eax"),
                64 => write!(f, "rax"),
                _ => panic!("Invalid register size"),
            },
            Register::R2(size) => match size {
                8 => write!(f, "cl"),
                16 => write!(f, "cx"),
                32 => write!(f, "ecx"),
                64 => write!(f, "rcx"),
                _ => panic!("Invalid register size"),
            },
            Register::R3(size) => match size {
                8 => write!(f, "dl"),
                16 => write!(f, "dx"),
                32 => write!(f, "edx"),
                64 => write!(f, "rdx"),
                _ => panic!("Invalid register size"),
            },
            Register::R4(size) => match size {
                8 => write!(f, "bl"),
                16 => write!(f, "bx"),
                32 => write!(f, "ebx"),
                64 => write!(f, "rbx"),
                _ => panic!("Invalid register size"),
            },
            Register::R5(size) => match size {
                8 => write!(f, "ah"),
                16 => write!(f, "sp"),
                32 => write!(f, "esp"),
                64 => write!(f, "rsp"),
                _ => panic!("Invalid register size"),
            },
            Register::R6(size) => match size {
                8 => write!(f, "ch"),
                16 => write!(f, "bp"),
                32 => write!(f, "ebp"),
                64 => write!(f, "rbp"),
                _ => panic!("Invalid register size"),
            },
            Register::R7(size) => match size {
                8 => write!(f, "dh"),
                16 => write!(f, "si"),
                32 => write!(f, "esi"),
                64 => write!(f, "rsi"),
                _ => panic!("Invalid register size"),
            },
            Register::R8(size) => match size {
                8 => write!(f, "bh"),
                16 => write!(f, "di"),
                32 => write!(f, "edi"),
                64 => write!(f, "rdi"),
                _ => panic!("Invalid register size"),
            },
        }
    }
}

pub enum TypeSize {
    Byte = 1,
    Word = 2,
    Double = 4,
    Quad = 8,
}

impl fmt::Display for TypeSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSize::Byte => write!(f, "byte"),
            TypeSize::Word => write!(f, "word"),
            TypeSize::Double => write!(f, "dword"),
            TypeSize::Quad => write!(f, "qword"),
        }
    }
}

impl BinaryOperator {
    pub fn get_instruction(&self) -> &str {
        match self {
            BinaryOperator::Add => "add",
            BinaryOperator::Sub => "sub",
            BinaryOperator::Mul => "imul",
            BinaryOperator::Div => todo!("Division instruction"),
            BinaryOperator::BitwiseOr => "or",
            BinaryOperator::BitwiseAnd => "and",
            BinaryOperator::BitwiseXor => "xor",
        }
    }
}

impl Local {
    pub fn get_word_type(&self) -> TypeSize {
        match self.size {
            1 => TypeSize::Byte,
            2 => TypeSize::Word,
            4 => TypeSize::Double,
            8 => TypeSize::Quad,
            _ => panic!("Unkown size"),
        }
    }
}

pub struct Compiler {
    filename: String,
    parser: Parser,
    buffer: Vec<u8>,
}

impl Compiler {
    pub fn from_file(filename: &str) -> Self {
        Self {
            filename: filename.to_owned(),
            parser: Parser::from_file(filename),
            buffer: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        self.parser.generate_tokens();

        let program = self.parser.generate_program();

        self.buffer.extend(self.write_program(&program));

        self.save_buffer();
    }

    fn write_program(&self, program: &Program) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend(format!("; Source File: {}", self.filename).as_bytes());

        buffer.extend("\nsection .text".as_bytes());
        buffer.extend("\n\tglobal _start".as_bytes());

        buffer.extend("\n_start:".as_bytes());
        buffer.extend("\n\tcall main".as_bytes());
        buffer.extend(format!("\n\tmov {}, {}", Register::R8(64), Register::R1(64)).as_bytes());
        buffer.extend(format!("\n\tmov {}, 0x3c", Register::R1(64)).as_bytes());
        buffer.extend("\n\tsyscall".as_bytes());

        for function in program.functions.iter() {
            buffer.extend(self.write_function(function, &program.functions));
        }

        buffer.push(b'\n');

        return buffer;
    }

    fn write_function(&self, function: &Function, functions: &Vec<Function>) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend(format!("\n{}:", function.name).as_bytes());

        let locals = &function.locals;

        // add 8 because future calls aligments
        let mut stack_size = locals.get_size() + 8;

        // force 16 bytes aligment
        stack_size += stack_size % 16;

        buffer.extend(format!("\n\tpush {}", Register::R6(64)).as_bytes());
        buffer.extend(format!("\n\tmov {}, {}", Register::R6(64), Register::R5(64)).as_bytes());

        buffer.extend(format!("\n\tsub {}, {:#x}", Register::R5(64), stack_size).as_bytes());

        for index in function.arguments.iter() {
            let argument = function.locals.get(*index).expect("Unreachable");

            buffer.extend(
                format!(
                    "\n\tmov {}, {} [{} + {:#x}]",
                    Register::R1(64),
                    argument.get_word_type(),
                    Register::R6(64),
                    16 + argument.offset
                )
                .as_bytes(),
            );

            buffer.extend(
                format!(
                    "\n\tmov {} [{} - {:#x}], {}\t; {}",
                    argument.get_word_type(),
                    Register::R6(64),
                    argument.offset + argument.size,
                    Register::R1(64),
                    argument.label,
                )
                .as_bytes(),
            );
        }

        buffer.extend(self.write_body(&function.name, &function.body, &function.locals, functions));

        buffer.extend(format!("\n.return_{}:", function.name).as_bytes());

        buffer.extend(format!("\n\tmov {}, {}", Register::R5(64), Register::R6(64)).as_bytes());
        buffer.extend(format!("\n\tpop {}", Register::R6(64)).as_bytes());

        buffer.extend(format!("\n\tret").as_bytes());

        return buffer;
    }

    fn write_body(
        &self,
        name: &str,
        body: &Scope,
        locals: &LocalStack,
        functions: &Vec<Function>,
    ) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        for statement in body.statements.iter() {
            match statement {
                Statement::Assign(local, expression) => {
                    let local = locals.get(*local).expect("Unreachable");

                    buffer.extend(self.write_expression(
                        expression,
                        &Register::R2(64),
                        &Register::R3(64),
                        locals,
                        functions,
                    ));

                    buffer.extend(
                        format!(
                            "\n\tmov {} [{} - {:#x}], {}\t; {}",
                            local.get_word_type(),
                            Register::R6(64),
                            local.offset + local.size,
                            Register::R2(64),
                            local.label
                        )
                        .as_bytes(),
                    );
                }
                Statement::Return(expression) => {
                    buffer.extend(self.write_expression(
                        expression,
                        &Register::R2(64),
                        &Register::R3(64),
                        locals,
                        functions,
                    ));

                    buffer.extend(
                        format!("\n\tmov {}, {}", Register::R1(64), Register::R2(64)).as_bytes(),
                    );

                    buffer.extend(format!("\n\tjmp .return_{}", name).as_bytes());
                }
                Statement::Call(expression) => {
                    // FIXME: idk
                    buffer.extend(self.write_expression(
                        expression,
                        &Register::R2(64),
                        &Register::R3(64),
                        locals,
                        functions,
                    ));
                }
            }
        }

        return buffer;
    }

    fn write_expression(
        &self,
        expression: &Expression,
        register: &Register,
        alt: &Register,
        locals: &LocalStack,
        functions: &Vec<Function>,
    ) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        match expression {
            Expression::Binary(binary_expression) => {
                let left = &*binary_expression.left;
                let right = &*binary_expression.right;

                if let Expression::Binary(_) = left {
                    buffer.extend(self.write_expression(left, register, alt, locals, functions));
                    buffer.extend(self.write_expression(right, alt, register, locals, functions));
                    buffer.extend(
                        format!(
                            "\n\t{} {}, {}",
                            binary_expression.operator.get_instruction(),
                            register,
                            alt
                        )
                        .as_bytes(),
                    );
                } else if let Expression::Binary(_) = right {
                    buffer.extend(self.write_expression(right, alt, register, locals, functions));
                    buffer.extend(self.write_expression(left, register, alt, locals, functions));
                    buffer.extend(
                        format!(
                            "\n\t{} {}, {}",
                            binary_expression.operator.get_instruction(),
                            register,
                            alt
                        )
                        .as_bytes(),
                    );
                } else {
                    buffer.extend(self.write_expression(left, register, alt, locals, functions));
                    buffer.extend(self.write_expression(right, alt, register, locals, functions));
                    buffer.extend(
                        format!(
                            "\n\t{} {}, {}",
                            binary_expression.operator.get_instruction(),
                            register,
                            alt
                        )
                        .as_bytes(),
                    );
                }
            }
            Expression::NumberLiteral(number) => {
                buffer.extend(format!("\n\tmov {}, {:#x}", register, number).as_bytes());
            }
            Expression::Local(index) => {
                if let Some(local) = locals.get(*index) {
                    buffer.extend(
                        format!(
                            "\n\tmov {}, {} [{} - {:#x}]\t; {}",
                            register,
                            local.get_word_type(),
                            Register::R6(64),
                            local.offset + local.size,
                            local.label
                        )
                        .as_bytes(),
                    );
                } else {
                    panic!("Unreachable");
                }
            }
            Expression::Call(index, expressions) => {
                let function = match functions.get(*index) {
                    Some(function) => function,
                    None => panic!("No function found"),
                };

                if function.arguments.len() != expressions.len() {
                    panic!("Argument mismath");
                }

                for (i, expression) in expressions.iter().enumerate() {
                    buffer.extend(self.write_expression(
                        expression,
                        &Register::R2(64),
                        &Register::R3(64),
                        locals,
                        functions,
                    ));

                    let argument = function
                        .locals
                        .get(*function.arguments.get(i).unwrap())
                        .unwrap();

                    buffer.extend(
                        format!("\n\tpush {};\t{}", Register::R2(64), argument.label).as_bytes(),
                    );
                }

                buffer.extend(format!("\n\tcall {}", function.name).as_bytes());
                buffer.extend(format!("\n\tmov {}, {}", register, Register::R1(64)).as_bytes());
            }
        }

        return buffer;
    }

    fn save_buffer(&self) {
        let path = Path::new(&self.filename);
        let stem = path.file_stem().expect("Error").to_str().unwrap();

        let mut file = File::create(format!("{}.s", stem)).expect("Can not create file");
        file.write(&self.buffer).expect("Can not write to file");

        Command::new("nasm")
            .arg("-felf64")
            .arg(format!("{}.s", stem))
            .arg("-o")
            .arg(format!("{}.o", stem))
            .output()
            .expect("failed to assemble");

        Command::new("ld")
            .arg(format!("{}.o", stem))
            .arg("-o")
            .arg(stem)
            .output()
            .expect("failed to link");
    }
}
