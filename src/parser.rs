use crate::lexer::{BinaryOperator, Lexer, Token, TokenType};

#[derive(Debug)]
pub struct Local {
    size: usize,
    offset: usize,
    label: String,
}

#[derive(Debug)]
pub struct LocalStack {
    locals: Vec<Local>,
}

impl LocalStack {
    fn new() -> Self {
        Self { locals: Vec::new() }
    }

    fn insert(&mut self, label: String, size: usize) -> usize {
        return match self.get(&label) {
            Some(index) => index,
            None => {
                let offset = match self.locals.last() {
                    Some(local) => local.offset + local.size,
                    None => 0,
                };

                self.locals.push(Local {
                    size,
                    offset,
                    label,
                });

                self.locals.len() - 1
            }
        };
    }

    fn get(&self, label: &str) -> Option<usize> {
        return self.locals.iter().position(|local| local.label == label);
    }
}

#[derive(Debug)]
pub struct Function {
    name: String,
    locals: LocalStack,
    arguments: Vec<usize>,
    body: Scope,
}

#[derive(Debug)]
pub struct Scope {
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Assign(usize, Expression),
    Return(Expression),
    Call(Expression),
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
    Local(usize),
    Call(usize, Vec<Expression>),
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

        let mut functions: Vec<Function> = Vec::new();

        while let Some(function) = self.next_function(&mut functions) {
            // TODO: Think about another way of storing functions
            functions.push(function);
        }

        program.functions = functions;

        return program;
    }

    fn next_function(&mut self, functions: &mut Vec<Function>) -> Option<Function> {
        if let Some(token) = self.next_token() {
            if let TokenType::Identifier(function_name) = token.token_type {
                self.next_colon();

                let mut locals = LocalStack::new();
                let arguments = self.next_args(&mut locals);
                let body = self.next_scope(&mut locals, functions);

                let function = Some(Function {
                    name: function_name,
                    locals,
                    arguments,
                    body,
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

    fn next_args(&mut self, locals: &mut LocalStack) -> Vec<usize> {
        self.next_l_par();

        let mut args: Vec<usize> = Vec::new();

        while let Some((label, size)) = self.next_arg() {
            let index = locals.insert(label, size);
            args.push(index);
        }

        self.next_r_par();

        return args;
    }

    fn next_arg(&mut self) -> Option<(String, usize)> {
        if let Some(token) = self.lookahead_token.clone() {
            match token.token_type {
                TokenType::Identifier(arg_name) => {
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

                    // FIXME: Don't hardcode local size
                    return Some((arg_name, 8));
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

    fn next_scope(&mut self, locals: &mut LocalStack, functions: &mut Vec<Function>) -> Scope {
        self.next_l_brace();

        let mut statements: Vec<Statement> = Vec::new();

        while let Some(statement) = self.next_statement(locals, functions) {
            statements.push(statement);
        }

        self.next_r_brace();

        return Scope { statements };
    }

    fn next_statement(
        &mut self,
        locals: &mut LocalStack,
        functions: &mut Vec<Function>,
    ) -> Option<Statement> {
        if let Some(token) = self.lookahead_token.clone() {
            match token.token_type {
                TokenType::Return => {
                    self.next_token();
                    return Some(self.next_return(locals, functions));
                }
                TokenType::Identifier(name) => {
                    self.next_token();

                    if let Some(token) = &self.lookahead_token {
                        match token.token_type {
                            TokenType::Equals => {
                                return Some(self.next_assign(name, locals, functions));
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
                            "{}:{}:{}: Expected statement but reached end of file.",
                            self.lexer.filename,
                            self.lexer.file_position.line,
                            self.lexer.file_position.column
                        );
                    }
                }
                TokenType::Call(_) => {
                    let call = self.next_call(locals, functions);
                    self.next_semicolon();
                    return Some(Statement::Call(call));
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

    fn next_assign(
        &mut self,
        identifier: String,
        locals: &mut LocalStack,
        functions: &mut Vec<Function>,
    ) -> Statement {
        self.next_equals();

        // FIXME: Don't hardcode local size
        let index = locals.insert(identifier, 8);

        let statement = Statement::Assign(index, self.next_expression(locals, functions, false));

        self.next_semicolon();

        return statement;
    }

    fn next_return(&mut self, locals: &LocalStack, functions: &mut Vec<Function>) -> Statement {
        let statement = Statement::Return(self.next_expression(locals, functions, false));

        self.next_semicolon();

        return statement;
    }

    fn next_call(&mut self, locals: &LocalStack, functions: &mut Vec<Function>) -> Expression {
        self.next_at();

        if let Some(token) = self.next_token() {
            if let TokenType::Identifier(function_name) = token.token_type {
                let index = match functions.iter().position(|f| f.name == function_name) {
                    Some(index) => index,
                    None => panic!(
                        "{}:{}:{}: Call to undefined function.",
                        self.lexer.filename, token.position.line, token.position.column
                    ),
                };

                let args = self.next_call_args(locals, functions);

                if args.len() != functions.get(index).unwrap().arguments.len() {
                    panic!(
                        "{}:{}:{}: Unmatched number of arguments.",
                        self.lexer.filename, token.position.line, token.position.column
                    );
                }

                return Expression::Call(index, args);
            } else {
                panic!(
                    "{}:{}:{}: Expected fuction name.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected function name but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_call_args(
        &mut self,
        locals: &LocalStack,
        functions: &mut Vec<Function>,
    ) -> Vec<Expression> {
        self.next_l_par();

        let mut expressions: Vec<Expression> = Vec::new();

        while let Some(arg) = self.next_call_arg(locals, functions) {
            expressions.push(arg);
        }

        self.next_r_par();

        return expressions;
    }

    fn next_call_arg(
        &mut self,
        locals: &LocalStack,
        functions: &mut Vec<Function>,
    ) -> Option<Expression> {
        if let Some(token) = &self.lookahead_token {
            match token.token_type {
                TokenType::RightPar => {
                    return None;
                }
                TokenType::Comma => {
                    if let Some(token_prev) = &self.current_token {
                        if let TokenType::LeftPar = token_prev.token_type {
                            panic!(
                                "{}:{}:{}: Expected a expression.",
                                self.lexer.filename, token.position.line, token.position.column
                            );
                        }
                    }

                    self.next_comma();
                    return Some(self.next_expression(locals, functions, true));
                }
                _ => {
                    return Some(self.next_expression(locals, functions, true));
                }
            }
        } else {
            panic!(
                "{}:{}:{}: Expected call arguments but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_expression(
        &mut self,
        locals: &LocalStack,
        functions: &mut Vec<Function>,
        call_arg: bool,
    ) -> Expression {
        let mut queue: Vec<Token> = Vec::new();

        let mut stack: Vec<Token> = Vec::new();

        let mut calls: Vec<Expression> = Vec::new();

        let mut last_token: Option<Token> = None;

        let mut end = false;

        while let Some(token) = self.lookahead_token.clone() {
            last_token = Some(token.clone());

            match &token.token_type {
                TokenType::Call(_) => {
                    let call = self.next_call(locals, functions);
                    calls.push(call);
                    queue.push(Token {
                        token_type: TokenType::Call(calls.len() - 1),
                        position: token.position,
                    });
                    continue;
                }
                TokenType::Identifier(_) => {
                    if let Some(current_token) = &self.current_token {
                        if let TokenType::Identifier(_) = current_token.token_type {
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
                    if stack.len() == 0 && call_arg {
                        end = true;
                        break;
                    }

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
                        if call_arg {
                            println!("tonoto 2");
                            end = true;
                            break;
                        }
                        panic!(
                            "{}:{}:{}: Unmatched parenthesis.",
                            self.lexer.filename, token.position.line, token.position.column
                        );
                    }
                }
                TokenType::Semicolon => {
                    if call_arg {
                        panic!(
                            "{}:{}:{}: Unexpected token.",
                            self.lexer.filename, token.position.line, token.position.column
                        );
                    }
                    end = true;
                    break;
                }
                TokenType::Comma => {
                    if !call_arg {
                        panic!(
                            "{}:{}:{}: Unexpected token.",
                            self.lexer.filename, token.position.line, token.position.column
                        );
                    }
                    end = true;
                    break;
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

        if end {
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
                    TokenType::Call(func) => {
                        if let Some(expr) = calls.get(*func) {
                            expressions.push(expr.clone());
                        } else {
                            panic!("Unreachable");
                        }
                    }
                    TokenType::NumberLiteral(number) => {
                        expressions.push(Expression::NumberLiteral(*number));
                    }
                    TokenType::Identifier(name) => {
                        let index = match locals.get(name) {
                            Some(index) => index,
                            None => {
                                panic!(
                                    "{}:{}:{}: Undeclared local.",
                                    self.lexer.filename, token.position.line, token.position.column
                                );
                            }
                        };
                        expressions.push(Expression::Local(index));
                    }
                    TokenType::BinaryOperation(operator) => {
                        if let (Some(right), Some(left)) = (expressions.pop(), expressions.pop()) {
                            expressions.push(Expression::Binary(BinaryExpression {
                                operator: operator.clone(),
                                left: Box::new(left),
                                right: Box::new(right),
                            }));
                        } else {
                            panic!(
                                "{}:{}:{}: Missing operator.",
                                self.lexer.filename, token.position.line, token.position.column
                            );
                        }
                    }
                    _ => {}
                }
            }

            if let Some(token) = last_token {
                if expressions.len() == 0 {
                    panic!(
                        "{}:{}:{}: Expected a expression.",
                        self.lexer.filename, token.position.line, token.position.column
                    );
                }
            } else {
                panic!("Unreachable");
            }

            assert!(expressions.len() == 1);

            return expressions.last().unwrap().to_owned();
        } else {
            panic!(
                "{}:{}:{}: Expected expression but found end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_at(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::Call(_) = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected a call token.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected a call token but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
    }

    fn next_equals(&mut self) {
        if let Some(token) = self.next_token() {
            if let TokenType::Equals = token.token_type {
                return;
            } else {
                panic!(
                    "{}:{}:{}: Expected an equals token.",
                    self.lexer.filename, token.position.line, token.position.column
                );
            }
        } else {
            panic!(
                "{}:{}:{}: Expected an equals token but reached end of file.",
                self.lexer.filename, self.lexer.file_position.line, self.lexer.file_position.column
            );
        }
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
