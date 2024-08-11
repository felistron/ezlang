use crate::lexer::{Lexer, Token, TokenType};

#[derive(Debug)]
pub struct Local {
    size: usize,
    label: String,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    arguments: Vec<Local>,
    locals: Vec<Local>,
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
                    locals: Vec::new(),
                });

                self.next_l_brace();

                self.next_r_brace();

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
