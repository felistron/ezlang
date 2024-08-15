use crate::lexer::{BinaryOperator, Lexer, Token, TokenType};

#[derive(Debug)]
pub struct Local {
    size: usize,
    label: String,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    arguments: Vec<Local>,
    body: Scope,
}

#[derive(Debug)]
pub struct Scope {
    locals: Vec<Local>,
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Assign(Local, Expression),
    If(Expression, Scope),
    While(Expression, Scope),
    For(Expression, Scope),
    Return(Expression),
    Expression,
    Call(String),
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    operator: BinaryOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    NumberLiteral(u64),
    Binary(BinaryExpression),
}

#[derive(Debug)]
pub struct Program {
    functions: Vec<Function>,
}

impl Program {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    tokens: Vec<Token>,
    position: usize,
    current_token: Option<Token>,
    lookahead_token: Option<Token>,
}

impl Parser {
    pub fn from_file(filename: &str) -> Self {
        return Self {
            lexer: Lexer::from_file(filename),
            tokens: Vec::new(),
            position: 0,
            current_token: None,
            lookahead_token: None,
        };
    }

    pub fn generate_tokens(&mut self) {
        while let Some(token) = self.lexer.next() {
            self.tokens.push(token);
        }

        if self.tokens.len() == 0 {
            panic!(
                "{}:{}:{}: Empty source file. Try writting a main function first.",
                self.lexer.filename, 1, 1
            );
        }
    }

    pub fn generate_program(&mut self) -> Program {
        return self.next_program();
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.position + 1 <= self.tokens.len() {
            if let Some(token) = self.tokens.get(self.position) {
                self.current_token = Some(token.to_owned());
            } else {
                self.current_token = None;
            }

            if let Some(token) = self.tokens.get(self.position + 1) {
                self.lookahead_token = Some(token.to_owned());
            } else {
                self.lookahead_token = None;
            }

            self.position += 1;
            return self.current_token.clone();
        } else {
            return None;
        }
    }

    fn next_program(&mut self) -> Program {
        let mut program = Program::new();

        while let Some(function) = self.next_function() {
            program.functions.push(function);
        }

        return program;
    }

    fn next_function(&mut self) -> Option<Function> {
        if let Some(token) = self.next_token() {
            if let TokenType::Identifier(function_name) = token.token_type {
                self.next_colon();

                let function = Some(Function {
                    name: function_name,
                    arguments: self.next_args(),
                    body: self.next_scope(),
                });

                return function;
            } else {
                panic!(
                    "{}:{}:{}: Expected function name",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        }

        return None;
    }

    fn next_args(&mut self) -> Vec<Local> {
        self.next_l_par();

        let mut args: Vec<Local> = Vec::new();

        while let Some(arg) = self.next_arg() {
            args.push(arg);
        }

        self.next_r_par();

        return args;
    }

    fn next_arg(&mut self) -> Option<Local> {
        if let Some(token) = self.lookahead_token.clone() {
            match token.token_type {
                TokenType::Identifier(arg_name) => {
                    let local = Some(Local {
                        size: 8, // TODO: Don't hardcode size of local
                        label: arg_name,
                    });

                    self.next_token();

                    if let Some(token) = self.lookahead_token.clone() {
                        match token.token_type {
                            TokenType::Comma => {
                                self.next_comma();
                            }
                            TokenType::RightPar => {}
                            TokenType::Identifier(_) => {
                                panic!("{}:{}:{}: Unexpected token. Maybe you forgot to put a comma between the two arguments.", self.lexer.filename, token.position.line, token.position.column);
                            }
                            _ => {
                                panic!(
                                    "{}:{}:{}: Unexpected token.",
                                    self.lexer.filename, token.position.line, token.position.column
                                );
                            }
                        }
                    } else {
                        panic!(
                            "{}:{}:{}: Expected comma or right parentheses but reached end of file.",
                            self.lexer.filename,
                            self.lexer.file_position.line,
                            self.lexer.file_position.column
                        );
                    }

                    return local;
                }
                TokenType::RightPar => {
                    if let Some(token) = self.current_token.clone() {
                        match token.token_type {
                            TokenType::Identifier(_) | TokenType::LeftPar => {
                                return None;
                            }
                            _ => {
                                panic!(
                                    "{}:{}:{}: Unexpected token",
                                    self.lexer.filename, token.position.line, token.position.column
                                );
                            }
                        }
                    } else {
                        panic!("Unreachable");
                    }
                }
                _ => {
                    panic!(
                        "{}:{}:{}: Expected right parentheses",
                        self.lexer.filename, token.position.line, token.position.column
                    );
                }
            }
        } else {
            panic!(
                "{}:{}:{}: Reached end of file",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_scope(&mut self) -> Scope {
        self.next_l_brace();

        let mut statements: Vec<Statement> = Vec::new();
        let locals: Vec<Local> = Vec::new();

        while let Some(statement) = self.next_statement() {
            statements.push(statement);
        }

        self.next_r_brace();

        return Scope { locals, statements };
    }

    fn next_statement(&mut self) -> Option<Statement> {
        if let Some(token) = self.lookahead_token.clone() {
            match token.token_type {
                TokenType::Return => {
                    self.next_token();
                    return Some(self.next_return());
                }
                TokenType::RightBrace => {
                    return None;
                }
                _ => {
                    panic!(
                        "{}:{}:{}: Unexpected token.",
                        self.lexer.filename, token.position.line, token.position.column
                    );
                }
            }
        } else {
            panic!(
                "{}:{}:{}: Expected statement but found end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_return(&mut self) -> Statement {
        let statement = Statement::Return(self.next_expression());

        self.next_semicolon();

        return statement;
    }

    fn next_expression(&mut self) -> Expression {
        let mut queue: Vec<Token> = Vec::new();

        let mut stack: Vec<Token> = Vec::new();

        while let Some(token) = self.lookahead_token.clone() {
            match &token.token_type {
                TokenType::NumberLiteral(_) => {
                    if let Some(current_token) = &self.current_token {
                        if let TokenType::NumberLiteral(_) = current_token.token_type {
                            panic!(
                                "{}:{}:{}: Invalid expression.",
                                self.lexer.filename, token.position.line, token.position.column
                            );
                        }
                    } else {
                        panic!("Unreachable");
                    }
                    queue.push(token);
                }
                TokenType::BinaryOperation(operator) => {
                    if let Some(current_token) = &self.current_token {
                        if let TokenType::BinaryOperation(_) = current_token.token_type {
                            panic!(
                                "{}:{}:{}: Invalid expression.",
                                self.lexer.filename, token.position.line, token.position.column
                            );
                        }
                    } else {
                        panic!("Unreachable");
                    }

                    let current_precedence = operator.get_precedence();

                    while let Some(token) = stack.last() {
                        match &token.token_type {
                            TokenType::BinaryOperation(operator) => {
                                let top_precedence = operator.get_precedence();

                                if top_precedence > current_precedence {
                                    queue.push(stack.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                            TokenType::LeftPar => {
                                break;
                            }
                            _ => {
                                panic!("Unreachable");
                            }
                        }
                    }

                    stack.push(token);
                }
                TokenType::LeftPar => {
                    stack.push(token);
                }
                TokenType::RightPar => {
                    let mut reached_left_par = false;

                    while let Some(token) = stack.pop() {
                        match &token.token_type {
                            TokenType::LeftPar => {
                                reached_left_par = true;
                                break;
                            }
                            TokenType::BinaryOperation(_) => queue.push(token),
                            _ => {
                                panic!("Unreachable");
                            }
                        }
                    }

                    if !reached_left_par {
                        panic!(
                            "{}:{}:{}: Unmatched parenthesis.",
                            self.lexer.filename, token.position.line, token.position.column
                        );
                    }
                }
                TokenType::Semicolon | TokenType::Comma => {
                    while let Some(token) = stack.pop() {
                        if let TokenType::LeftPar | TokenType::RightPar = token.token_type {
                            panic!(
                                "{}:{}:{}: Unmatched parentheses.",
                                self.lexer.filename, token.position.line, token.position.column
                            );
                        }
                        queue.push(token);
                    }

                    let mut expressions: Vec<Expression> = Vec::new();

                    for token in queue.iter() {
                        match &token.token_type {
                            TokenType::NumberLiteral(number) => {
                                expressions.push(Expression::NumberLiteral(*number));
                            }
                            TokenType::BinaryOperation(operator) => {
                                if let (Some(right), Some(left)) =
                                    (expressions.pop(), expressions.pop())
                                {
                                    expressions.push(Expression::Binary(BinaryExpression {
                                        operator: operator.clone(),
                                        left: Box::new(left),
                                        right: Box::new(right),
                                    }));
                                } else {
                                    panic!(
                                        "{}:{}:{}: Missing operator.",
                                        self.lexer.filename,
                                        token.position.line,
                                        token.position.column
                                    );
                                }
                            }
                            _ => {}
                        }
                    }

                    if expressions.len() == 0 {
                        panic!(
                            "{}:{}:{}: Missing expression.",
                            self.lexer.filename, token.position.line, token.position.column
                        );
                    }

                    assert!(expressions.len() == 1);

                    return expressions.last().unwrap().to_owned();
                }
                _ => {
                    panic!(
                        "{}:{}:{}: Unexpected token.",
                        self.lexer.filename, token.position.line, token.position.column
                    );
                }
            };

            self.next_token();
        }

        panic!(
            "{}:{}:{}: Expected expression but found end of file.",
            self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
        );
    }

    fn next_semicolon(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::Semicolon = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected a semicolon.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected a semicolon but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_comma(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::Comma = token.token_type {
                return;
            } else {
                panic!("Expected comma token.");
            }
        } else {
            panic!("No token");
        }
    }

    fn next_colon(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::Colon = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected a colon after function name.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected a colon after function name but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_r_brace(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::RightBrace = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected right brace token.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected right brace token but reached end of file",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_l_brace(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::LeftBrace = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected left brace token.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected left brace token but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_r_par(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::RightPar = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected right parentheses token.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected right parentheses token but reached end of file",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_l_par(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::LeftPar = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected left parentheses token.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected left parentheses token but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }
}
