use core::fmt;
use std::{collections::HashMap, fs::read_to_string, process::{Command, Stdio}};

use regex::Regex;

#[derive(Debug, Clone)]
pub struct SyntaxError {
    position: Position,
    message: String,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Syntax error at {}: {}", self.position, self.message)
    }
}


#[derive(Debug, Clone)]
pub struct CompileError {
    message: String,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Compilation error: {}", self.message)
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone)]
pub struct Position {
    file: String,
    line: usize,
    column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Symbol(Position, String),
    NumberLiteral(Position, u32),
    StringLiteral(Position, String),
    Equal(Position),
    OpenParenthesis(Position),
    CloseParenthesis(Position),
    OpenBracket(Position),
    CloseBracket(Position),
    Comma(Position),
    Semicolon(Position),
    BinaryOperator(Position, BinaryOperator),
    Colon(Position),
    Return(Position),
    Assembly(Position),
    EOF(Position),
}

impl Token {
    pub fn get_position(&self) -> &Position {
        return match self {
            Token::Symbol(position, _) => position,
            Token::NumberLiteral(position, _) => position,
            Token::StringLiteral(position, _) => position,
            Token::Equal(position) => position,
            Token::OpenParenthesis(position) => position,
            Token::CloseParenthesis(position) => position,
            Token::OpenBracket(position) => position,
            Token::CloseBracket(position) => position,
            Token::Comma(position) => position,
            Token::Semicolon(position) => position,
            Token::BinaryOperator(position, _) => position,
            Token::Colon(position) => position,
            Token::Return(position) => position,
            Token::Assembly(position) => position,
            Token::EOF(position) => position,
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    NumberLiteral(u32),
    StringLiteral(String, String),
    Symbol(String),
    CallStatement(String, Option<Vec<Expression>>),
    BinaryOperation(BinaryOperator, Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub struct Argument {
    name: String,
}

#[derive(Debug)]
pub enum Statement {
    AssignStatement { symbol_name: String, expression: Expression },
    CallStatement { function_name: String, arguments: Option<Vec<Expression>> },
    ReturnStatement { return_value: Expression },
    // TODO: Assembly statement
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    function_name: String,
    arguments: Vec<Argument>,
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct Program {
    functions: Vec<FunctionDeclaration>,
    string_literals: HashMap<String, String>,
}

impl Program {
    pub fn new() -> Self {
        Program { functions: Vec::new(), string_literals: HashMap::new() }
    }
}

pub struct Parser {
    tokenizer: Tokenizer,
    lookahead: Option<Token>,
    string_literals: HashMap<String, String>,
}

impl Parser {
    pub fn new(filename: String, source_code: String) -> Self {
        Parser {
            tokenizer: Tokenizer::new(filename, source_code),
            lookahead: None,
            string_literals: HashMap::new(),
        }
    }

    fn consume(&mut self) -> Option<Token> {
        let current = self.lookahead.clone();
        self.lookahead = self.tokenizer.next();
        return current;
    }

    pub fn parse(&mut self) -> Result<Program, SyntaxError> {
        self.lookahead = self.tokenizer.next();
        return self.program();
    }

    fn program(&mut self) -> Result<Program, SyntaxError> {
        let mut program = Program::new();

        while let Some(_) = &self.lookahead {
            if let Some(Token::EOF(_)) = &self.lookahead {
                self.consume();
                break;
            }

            match self.function() {
                Ok(function) => program.functions.push(function),
                Err(error) => return Err(error),
            }
        }

        if program.functions.len() < 1 {
            return Err(
                SyntaxError {
                    position: Position {
                        file: self.tokenizer.file.to_owned(),
                        line: 0,
                        column: 0
                    },
                    message: "Program cannot be empty".to_owned()
                }
            );
        }

        if !program.functions.iter().any(|func| func.function_name == "main") {
            return Err(
                SyntaxError {
                    position: Position {
                        file: self.tokenizer.file.to_owned(),
                        line: 0,
                        column: 0
                    },
                    message: "No entry point: Missing main function".to_owned()
                }
            );
        }

        for (label, string) in &self.string_literals {
            program.string_literals.insert(label.to_owned(), string.to_owned());
        }

        println!("{:#?}", &program);

        return Ok(program);
    }

    fn function(&mut self) -> Result<FunctionDeclaration, SyntaxError> {
        let token = self.consume();

        if let Some(Token::Symbol(position, function_name)) = token {
            let token = self.consume();

            if let Some(Token::Colon(_)) = token {

                let arguments = match self.argument_list_declaration() {
                    Ok(arguments) => arguments,
                    Err(err) => return Err(err)
                };

                let statements = self.function_body();

                return Ok(FunctionDeclaration {
                    function_name,
                    arguments,
                    statements,
                });
            } else {
                let position = Position {
                    file: position.file,
                    line: position.line,
                    column: position.column + function_name.len() 
                };

                return Err(
                    SyntaxError {
                        position,
                        message: "Missing token at function declaration".to_owned()
                    }
                );
            }
        } else {
            let token = token.unwrap();
            let position = token.get_position();

            return Err(
                SyntaxError {
                    position: position.to_owned(),
                    message: "Unexpected token at function declaration".to_owned()
                }
            );
        }
    }

    fn statement(&mut self) -> Statement {
        let token = self.consume();

        if let Some(Token::Symbol(_, symbol_name)) = token.clone() {
            //let token = self.consume();

            if let Some(Token::Equal(_)) = &self.lookahead {
                self.consume();
                
                return Statement::AssignStatement {
                    symbol_name,
                    expression: self.expression()
                };
            } else if let Some(Token::OpenParenthesis(_)) = &self.lookahead {
                return Statement::CallStatement {
                    function_name: symbol_name,
                    arguments: self.argument_list_call(),
                };
            } else {
                let token = token.unwrap();
                let position = token.get_position();

                panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
            }
        } else if let Some(Token::Return(_)) = token.clone() {
            return Statement::ReturnStatement { return_value: self.expression() }
        } else {
            let token = token.unwrap();
            let position = token.get_position();

            panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
        }
    }

    fn argument_list_declaration(&mut self) -> Result<Vec<Argument>, SyntaxError> {
        let token = self.consume();
        
        if let Some(Token::OpenParenthesis(_)) = &token {
            let mut arguments: Vec<Argument> = Vec::new();

            if let Some(Token::CloseParenthesis(_)) = &self.lookahead {
                self.consume();
            } else if let Some(Token::Symbol(_, _)) = &self.lookahead {
                while let Some(Token::Symbol(_, argument_name)) = self.consume() {
                    arguments.push(Argument { name: argument_name });

                    let token = self.consume();

                    if let Some(Token::Comma(_)) = token {
                        continue;
                    } else if let Some(Token::CloseParenthesis(_)) = token {
                        break;
                    } else {
                        let token = token.unwrap();
                        let position = token.get_position().to_owned();

                        return Err(
                            SyntaxError {
                                position: position,
                                message: "No se weey".to_owned()
                            }
                        );
                    }
                }
            } else {
                let token = token.unwrap();
                let position = token.get_position().to_owned();

                return Err(
                    SyntaxError {
                        position,
                        message: "Unexpected token at function's close argument list declaration".to_owned()
                    }
                );
            }
            
            return Ok(arguments);
        } else {
            let token = token.unwrap();
            let position = token.get_position().to_owned();

            return Err(
                SyntaxError {
                    position,
                    message: "Unexpected token at function's argument list declaration".to_owned()
                }
            );
        }
    }

    fn argument_list_call(&mut self) -> Option<Vec<Expression>> {
        let token = self.consume();

        if let Some(Token::OpenParenthesis(_)) = token {
            let mut arguments: Vec<Expression> = Vec::new();

            loop {
                if let Some(Token::CloseParenthesis(_)) = &self.lookahead {
                    break;
                }
                
                arguments.push(self.expression());

                let token = self.consume();

                if let Some(Token::Comma(_)) = token {
                    continue;
                } else if let Some(Token::CloseParenthesis(_)) = token {
                    break;
                } else {
                    let token = token.unwrap();
                    let position = token.get_position();

                    panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);   
                }
            };

            return Some(arguments);
        } else {
            let token = token.unwrap();
            let position = token.get_position();

            panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
        }
    }

    fn function_body(&mut self) -> Vec<Statement> {
        let token = self.consume();

        if let Some(Token::OpenBracket(_)) = token.clone() {
            let mut statements: Vec<Statement> = Vec::new();

            loop {
                if let Some(Token::CloseBracket(_)) = &self.lookahead {
                    self.consume();
                    break;
                }

                statements.push(
                    self.statement()
                );

                if let Some(Token::Semicolon(_)) = self.consume() {
                    continue;
                } else {
                    let token = token.unwrap();
                    let position = token.get_position();

                    panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
                }
            };

            return statements;
        } else {
            let token = token.unwrap();
            let position = token.get_position();

            panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
        }
    }

    fn expression(&mut self) -> Expression {
        let token = self.consume();

        if let Some(Token::BinaryOperator(_, operator)) = token.clone() {
            let token = self.consume();

            if let Some(Token::NumberLiteral(_, value)) = token {
                return Expression::BinaryOperation(
                    operator,
                    Box::new(Expression::NumberLiteral(value)),
                    Box::new(self.expression())
                );
            } else if let Some(Token::Symbol(_, symbol_name)) = token {
                return Expression::BinaryOperation(
                    operator,
                    Box::new(Expression::Symbol(symbol_name)),
                    Box::new(self.expression())
                );
            } else {
                let token = token.unwrap();
                let position = token.get_position();

                panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
            }
        } else if let Some(Token::NumberLiteral(_, value)) = token {
            return Expression::NumberLiteral(value);
        } else if let Some(Token::StringLiteral(_, string)) = token {
            let label = format!("strltr.{}", self.string_literals.len());
            self.string_literals.insert(label.to_string(), string.clone());
            return Expression::StringLiteral(label, string.clone());
        } else if let Some(Token::Symbol(_, symbol_name)) = token {
            if let Some(Token::OpenParenthesis(_)) = &self.lookahead {
                return Expression::CallStatement(
                    symbol_name,
                    self.argument_list_call(),
                );
            }

            return Expression::Symbol(symbol_name);
        } else {
            let token = token.unwrap();
            let position = token.get_position();

            panic!("Syntax error: {}:{}:{}", position.file, position.line, position.column);
        }
    }
}

struct Tokenizer {
    file: String,
    buffer: String,
    cursor: usize,
    line: usize,
    column: usize,
    reached_eof: bool,
}

impl Tokenizer {
    fn new(filename: String, source_code: String) -> Self {
        return Tokenizer {
            file: filename,
            buffer: source_code,
            cursor: 0,
            line: 1,
            column: 1,
            reached_eof: false,
        }
    }

    fn has_next(&self) -> bool {
        self.cursor < self.buffer.len()
    }

    fn next(&mut self) -> Option<Token> {
        if self.reached_eof {
            return None;
        }
        
        let current_position: Position = Position {
            file: self.file.to_string(),
            line: self.line,
            column: self.column
        };

        if !self.has_next() {
            self.reached_eof = true;
            return Some(Token::EOF(current_position));
        }

        // White space:
        let re = Regex::new(r"^[\s]").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(matched_string) = captures.get(0) {
                self.cursor += 1;
                self.column += 1;

                if matched_string.as_str() == "\n" {
                    self.line += 1;
                    self.column = 1;
                }
                
                return self.next();
            }
        }

        // Number:
        let re = Regex::new(r"^[-+]?\d+").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(matched_string) = captures.get(0) {
                let number_literal: u32  = matched_string.as_str().parse().unwrap();
                self.cursor += matched_string.len();
                self.column += matched_string.len();
                
                return Some(Token::NumberLiteral(current_position, number_literal));
            }
        }

        // Binary Operator
        let re = Regex::new(r"^[+\-*/&|^%]").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(matched_string) = captures.get(0) {
                self.cursor += 1;
                self.column += 1;

                return match matched_string.as_str() {
                    "+" => Some(Token::BinaryOperator(current_position, BinaryOperator::Addition)),
                    "-" => Some(Token::BinaryOperator(current_position, BinaryOperator::Subtraction)),
                    "*" => Some(Token::BinaryOperator(current_position, BinaryOperator::Multiplication)),
                    "/" => Some(Token::BinaryOperator(current_position, BinaryOperator::Division)),
                    "&" => Some(Token::BinaryOperator(current_position, BinaryOperator::And)),
                    "|" => Some(Token::BinaryOperator(current_position, BinaryOperator::Or)),
                    "^" => Some(Token::BinaryOperator(current_position, BinaryOperator::Xor)),
                    _ => { panic!("Unkown binary operator"); }
                }
            }
        }

        // Equals symbol
        let re = Regex::new(r"^[=]").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(_) = captures.get(0) {
                self.cursor += 1;
                self.column += 1;

                return Some(Token::Equal(current_position));
            }
        }

        // Semicolon
        let re = Regex::new(r"^[;]").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(_) = captures.get(0) {
                self.cursor += 1;
                self.column += 1;
                
                return Some(Token::Semicolon(current_position));
            }
        }

        // Symbol
        let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(matched_string) = captures.get(0) {
                self.cursor += matched_string.len();
                self.column += matched_string.len();

                if matched_string.as_str() == "return" {
                    return Some(Token::Return(current_position));
                } else if matched_string.as_str() == "asm" {
                    return Some(Token::Assembly(current_position));
                } else {
                    return Some(Token::Symbol(current_position, String::from(matched_string.as_str())));
                }
            }
        }

        // Special characters
        let re = Regex::new(r"^[{}():,]").unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(matched_string) = captures.get(0) {
                self.cursor += matched_string.len();
                self.column += matched_string.len();

                return match matched_string.as_str() {
                    "(" => Some(Token::OpenParenthesis(current_position)),
                    ")" => Some(Token::CloseParenthesis(current_position)),
                    "{" => Some(Token::OpenBracket(current_position)),
                    "}" => Some(Token::CloseBracket(current_position)),
                    ":" => Some(Token::Colon(current_position)),
                    "," => Some(Token::Comma(current_position)),
                    _ => { panic!("Unkown token"); },
                }
            }
        }

        // Strings
        let re = Regex::new(r#"^"([^"\\]*(\\.[^"\\]*)*)""#).unwrap();

        if let Some(captures) = re.captures(&self.buffer[self.cursor..]) {
            if let Some(matched_string) = captures.get(0) {
                self.cursor += matched_string.len();
                self.column += matched_string.len();

                let matched_string = matched_string.as_str().trim_matches('"');

                return Some(Token::StringLiteral(current_position, matched_string.to_string()));
            }
        }

        panic!("Syntax error: {}:{}:{}", current_position.file, current_position.line, current_position.column);
    }
}

pub struct Compiler {
    filename: String,
}

impl Compiler {
    pub fn new(input_filename: &str) -> Self {
        Compiler {
            filename: input_filename.to_owned(),
        }
    }

    pub fn compile(&mut self) -> Result<(), CompileError> {
        let source_code = match read_to_string(self.filename.to_owned()) {
            Ok(buffer) => buffer,
            Err(_) => return Err(CompileError { message: format!("Cannot open {}", self.filename)}),
        };

        let assembly_code = match self.assemble(source_code) {
            Ok(assembly) => assembly,
            Err(error) => return Err(error),
        };

        let file_name = self.filename.split(".ez").collect::<Vec<&str>>()[0];

        let assembly_filename = format!("./build/{}.asm", file_name);
        let link_filename = format!("./build/{}.o", file_name);
        let exec_filename = format!("./build/{}", file_name);

        let _ = std::fs::write(&assembly_filename, assembly_code);

        let output = Command::new("nasm")
            .args(["-f", "elf64"])
            .args(["-o", &link_filename])
            .arg(&assembly_filename)
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        if !output.status.success() {
            return Err(CompileError { message: format!("Assemble error\n\t{}", String::from_utf8(output.stderr).unwrap()) });
        }

        let output = Command::new("ld")
            .args(["-o", &exec_filename])
            .arg(&link_filename)
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        if !output.status.success() {
            return Err(CompileError { message: format!("Link error\n\t{}", String::from_utf8(output.stderr).unwrap()) });
        }

        return Ok(());
    }

    fn assemble(&mut self, source_code: String) -> Result<String, CompileError> {
        let mut parser: Parser = Parser::new(self.filename.to_owned(), source_code);

        let program = match parser.parse() {
            Ok(program) => program,
            Err(error) => return Err(CompileError { message: error.to_string() }),
        };

        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend(self.program(&program));

        return Ok(String::from_utf8(buffer).unwrap())
    }

    fn program(&self, program: &Program) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend(format!("; Code generated with ezlang").as_bytes());
        buffer.extend(format!("\n; filename: {}", self.filename).as_bytes());

        buffer.extend("\n\nsection .data".as_bytes());

        for (label, string) in &program.string_literals {
            buffer.extend(format!("\n    {} db `{}`, 0", label, string).as_bytes());
            buffer.extend(format!("\n    {}.len equ $-{}", label, label).as_bytes());
        }

        buffer.extend("\n\nsection .text".as_bytes());
        buffer.extend("\n    global _start".as_bytes());

        buffer.extend("\n\nwrite:".as_bytes());
        buffer.extend("\n    ; prologue".as_bytes());
        buffer.extend("\n    push rbp".as_bytes());
        buffer.extend("\n    mov rbp, rsp".as_bytes());
        buffer.extend("\n    mov rax, 0x1".as_bytes());
        buffer.extend("\n    syscall".as_bytes());
        buffer.extend("\n    ; epilogue".as_bytes());
        buffer.extend("\n    mov rsp, rbp".as_bytes());
        buffer.extend("\n    pop rbp".as_bytes());
        buffer.extend("\n    ret".as_bytes());

        for function in &program.functions {
            buffer.extend(self.function_declaration(&function));
        }

        buffer.extend("\n\n_start:".as_bytes());
        buffer.extend("\n    call main ; entry point".as_bytes());
        buffer.extend("\n    mov rdi, rax".as_bytes());
        buffer.extend("\n    mov rax, 60".as_bytes());
        buffer.extend("\n    syscall".as_bytes());

        return buffer;
    } 

    fn function_declaration(&self, function: &FunctionDeclaration) -> Vec<u8> {
        let mut locals: HashMap<&str, usize> = HashMap::new();
        let mut offset: usize = 0;

        let mut function_body: Vec<u8> = Vec::new();

        let regs = ["edi", "esi", "edx", "ecx"];
        let mut count: usize = 0;

        for argument in &function.arguments {
            offset += 4;
            locals.insert(argument.name.as_str(), offset);
            function_body.extend(format!("\n    mov dword[rbp - {}], {} ; arg: {}", offset, regs[count], argument.name).as_bytes());
            count += 1;
        }

        for statement in &function.statements {
            match statement {
                Statement::AssignStatement { symbol_name, expression } => {
                    if let Some(_) = locals.get(symbol_name.as_str()) {

                    } else {
                        offset += 4;
                        locals.insert(&symbol_name.as_str(), offset);
                    }
                    
                    function_body.extend(self.expression(expression, "eax", &locals));
                    function_body.extend(format!("\n    mov dword[rbp - {}], eax ; var: {}", offset, symbol_name).as_bytes());
                },
                Statement::ReturnStatement { return_value } => {
                    function_body.extend(self.expression(return_value, "eax", &locals));
                    function_body.extend(format!("\n    jmp .{}.ret", function.function_name).as_bytes());
                },
                Statement::CallStatement { function_name, arguments } => {
                    if let Some(arguments) = arguments {
                        let regs = ["edi", "esi", "edx", "ecx"];
                        let mut count: usize = 0;
    
                        for argument in arguments {
                            function_body.extend(self.expression(argument, "eax", &locals));
                            function_body.extend(format!("\n    mov {}, eax", regs[count]).as_bytes());
                            count += 1;
                        }
                    }
                    function_body.extend(format!("\n    call {}", function_name).as_bytes());
                },
            }
        }

        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend(format!("\n\n{}:", function.function_name).as_bytes());
        buffer.extend("\n    ; prologue".as_bytes());
        buffer.extend("\n    push rbp".as_bytes());
        buffer.extend("\n    mov rbp, rsp".as_bytes());
        buffer.extend(format!("\n    sub rsp, {}", offset + (8 - (offset % 8))).as_bytes());

        buffer.extend(function_body);

        buffer.extend("\n    ; epilogue".as_bytes());
        buffer.extend(format!("\n.{}.ret:", function.function_name).as_bytes());
        buffer.extend("\n    mov rsp, rbp".as_bytes());
        buffer.extend("\n    pop rbp".as_bytes());
        buffer.extend("\n    ret".as_bytes());

        return buffer;
    }

    fn expression(&self, expression: &Expression, reg: &str, locals: &HashMap<&str, usize>) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        match expression {
            Expression::NumberLiteral(value) => {
                buffer.extend(format!("\n    mov {}, {:#x}", reg, value).as_bytes());
            },
            Expression::Symbol(symbol_name) => {
                let offset = locals.get(symbol_name.as_str()).expect(format!("Runtime error: Unknown symbol \"{}\"", symbol_name).as_str());
                buffer.extend(format!("\n    mov {}, dword[rbp - {}] ; var: {}", reg, offset, symbol_name).as_bytes());
            },
            Expression::CallStatement(function_name, arguments) => {
                if let Some(arguments) = arguments {
                    let regs = ["edi", "esi", "edx", "ecx"];
                    let mut count: usize = 0;

                    for argument in arguments {
                        buffer.extend(self.expression(argument, "eax", locals));
                        buffer.extend(format!("\n    mov {}, eax", regs[count]).as_bytes());
                        count += 1;
                    }
                }
                buffer.extend(format!("\n    call {}", function_name).as_bytes());
            },
            Expression::BinaryOperation(operator, left, right) => {
                buffer.extend(self.expression(&right, "eax", locals));
                buffer.extend(self.expression(&left, "edi", locals));
                
                let op = match operator {
                    BinaryOperator::Addition => "add",
                    BinaryOperator::Subtraction => "sub",
                    BinaryOperator::Multiplication => "imul",
                    BinaryOperator::Division => "idiv",
                    BinaryOperator::And => "and",
                    BinaryOperator::Or => "or",
                    BinaryOperator::Xor => "xor",
                };

                buffer.extend(format!("\n    {} eax, edi", op).as_bytes());
            }
            Expression::StringLiteral(label, string) => {
                buffer.extend(format!("\n    mov {}, {} ; str: \"{}\"", reg, label, string).as_bytes());
            },
        }

        return buffer;
    }
}