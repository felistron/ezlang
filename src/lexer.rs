use core::panic;
use std::{fs::File, io::Read};

#[derive(Debug, Clone)]
pub struct Position {
    line: usize,
    column: usize,
}

impl Position {
    pub fn start() -> Self {
        Self { line: 1, column: 1 }
    }

    pub fn new_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    pub fn next_column(&mut self) {
        self.column += 1;
    }
}

pub struct Lexer {
    filename: String,
    data: Vec<u8>,
    position: usize,
    current_char: u8,
    reached_eof: bool,
    file_position: Position,
}

#[derive(Debug)]
pub enum TokenType {
    NumberLiteral(u64),
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    position: Position,
}

impl Lexer {
    pub fn from_file(filename: &str) -> Self {
        let mut file: File = File::open(filename).expect("File does not exists");

        let mut buf: Vec<u8> = Vec::new();

        file.read_to_end(&mut buf).expect("Could not read file");

        return Self {
            filename: filename.to_owned(),
            current_char: buf[0],
            data: buf,
            position: 0,
            reached_eof: false,
            file_position: Position::start(),
        };
    }

    pub fn next(&mut self) -> Option<Token> {
        self.skip_whitespaces();

        if self.reached_eof {
            return None;
        }

        return match self.current_char {
            b'0'..b'9' => Some(self.read_number_like()),
            _ => {
                panic!(
                    "{}:{}:{}: Unkown token",
                    self.filename, self.file_position.line, self.file_position.column
                );
            }
        };
    }

    fn next_char(&mut self) -> u8 {
        if self.current_char == b'\n' {
            self.file_position.new_line();
        } else {
            self.file_position.next_column();
        }

        if self.position + 1 < self.data.len() {
            self.position += 1;
            self.current_char = self.data[self.position];
        } else {
            self.reached_eof = true;
        }

        return self.current_char;
    }

    fn skip_whitespaces(&mut self) {
        let mut c = self.current_char;

        while (c as char).is_whitespace() && !self.reached_eof {
            c = self.next_char();
        }
    }

    fn read_number_like(&mut self) -> Token {
        let current_position = self.file_position.clone();

        let base = self.next_decimal();

        if self.current_char == b'#' {
            self.next_char();
            let number = match base {
                2 => self.next_binary(),
                8 => self.next_octal(),
                10 => self.next_decimal(),
                16 => self.next_hexadecimal(),
                _ => panic!("Unkown numerical base"),
            };

            return Token {
                token_type: TokenType::NumberLiteral(number),
                position: current_position,
            };
        } else {
            return Token {
                token_type: TokenType::NumberLiteral(base),
                position: current_position,
            };
        }
    }

    fn next_binary(&mut self) -> u64 {
        let mut result: u64 = 0;

        let mut c = self.current_char;

        while (c as char).is_alphanumeric() && !self.reached_eof {
            if c == b'0' || c == b'1' {
                result = result * 2 + (c - b'0') as u64;
            } else {
                panic!(
                    "{}:{}:{}: Invalid binary number",
                    self.filename, self.file_position.line, self.file_position.column
                );
            }
            c = self.next_char();
        }

        return result;
    }

    fn next_octal(&mut self) -> u64 {
        let mut result: u64 = 0;

        let mut c = self.current_char;

        while (c as char).is_alphanumeric() && !self.reached_eof {
            if c >= b'0' && c <= b'7' {
                result = result * 8 + (c - b'0') as u64;
            } else {
                panic!(
                    "{}:{}:{}: Invalid octal number",
                    self.filename, self.file_position.line, self.file_position.column
                );
            }
            c = self.next_char();
        }

        return result;
    }

    fn next_hexadecimal(&mut self) -> u64 {
        let mut result: u64 = 0;

        let mut c = self.current_char;

        while (c as char).is_alphanumeric() && !self.reached_eof {
            let value = match c {
                b'0'..b'9' => c - b'0',
                b'A'..b'F' => 10 + c - b'A',
                b'a'..b'f' => 10 + c - b'a',
                _ => {
                    panic!(
                        "{}:{}:{}: Invalid octal number",
                        self.filename, self.file_position.line, self.file_position.column
                    );
                }
            };

            result = result * 16 + value as u64;
            c = self.next_char();
        }

        return result;
    }

    fn next_decimal(&mut self) -> u64 {
        let mut result: u64 = 0;

        let mut c = self.current_char;

        while (c as char).is_alphanumeric() && !self.reached_eof {
            if (c as char).is_numeric() {
                result = result * 10 + (c - b'0') as u64;
            } else {
                panic!(
                    "{}:{}:{}: Invalid decimal number",
                    self.filename, self.file_position.line, self.file_position.column
                );
            }
            c = self.next_char();
        }

        return result;
    }
}
