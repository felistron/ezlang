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
    StringLiteral(String),
    Character(char),
    Identifier(String),
    Return,
    If,
    While,
    For,
    True,
    False,
    Colon,
    Semicolon,
    LeftPar,
    RightPar,
    LeftBrace,
    RightBrace,
    BinaryAdd,
    BinarySub,
    Equals,
    BinaryDiv,
    BinaryMul,
    Comma,
    BinaryAnd,
    BinaryOr,
    BinaryXor,
    UnaryNot,
    UnaryInc,
    UnaryDec,
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
            b':' => Some(self.read_colon()),
            b'(' => Some(self.read_l_par()),
            b')' => Some(self.read_r_par()),
            b'{' => Some(self.read_l_brace()),
            b'}' => Some(self.read_r_brace()),
            b';' => Some(self.read_semicolon()),
            b'+' => Some(self.read_add()),
            b'-' => Some(self.read_sub()),
            b'=' => Some(self.read_equals()),
            b'/' => Some(self.read_div()),
            b'*' => Some(self.read_mul()),
            b',' => Some(self.read_comma()),
            b'&' => Some(self.read_and()),
            b'|' => Some(self.read_or()),
            b'^' => Some(self.read_xor()),
            b'!' => Some(self.read_not()),
            b'0'..=b'9' => Some(self.read_number_like()),
            b'a'..=b'z' | b'A'..b'Z' | b'_' => Some(self.read_identifier()),
            b'"' => Some(self.read_string()),
            b'\'' => Some(self.read_character()),
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

    fn read_not(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::UnaryNot,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_xor(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::BinaryXor,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_or(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::BinaryOr,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_and(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::BinaryAnd,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_div(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::BinaryDiv,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_mul(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::BinaryMul,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_comma(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::Comma,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_equals(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::Equals,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_sub(&mut self) -> Token {
        let current_position = self.file_position.clone();

        let c = self.next_char();

        return if c == b'-' {
            self.next_char();

            Token {
                token_type: TokenType::UnaryDec,
                position: current_position,
            }
        } else {
            Token {
                token_type: TokenType::BinarySub,
                position: self.file_position.clone(),
            }
        };
    }

    fn read_add(&mut self) -> Token {
        let current_position = self.file_position.clone();

        let c = self.next_char();

        return if c == b'+' {
            self.next_char();

            Token {
                token_type: TokenType::UnaryInc,
                position: current_position,
            }
        } else {
            Token {
                token_type: TokenType::BinaryAdd,
                position: self.file_position.clone(),
            }
        };
    }

    fn read_r_brace(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::RightBrace,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_l_brace(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::LeftBrace,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_r_par(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::RightPar,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_l_par(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::LeftPar,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_semicolon(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::Semicolon,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_colon(&mut self) -> Token {
        let token = Token {
            token_type: TokenType::Colon,
            position: self.file_position.clone(),
        };
        self.next_char();
        return token;
    }

    fn read_character(&mut self) -> Token {
        let current_position = self.file_position.clone();

        let mut c = self.next_char();

        if c == b'\\' {
            match self.next_char() {
                b'\'' => c = b'\'',
                b'n' => c = b'\n',
                b't' => c = b'\t',
                b'r' => c = b'\r',
                b'0' => c = b'\0',
                b'\\' => c = b'\\',
                _ => {}
            }
        }

        if self.next_char() != b'\'' {
            panic!(
                "{}:{}:{}: Expected closing character sign",
                self.filename, current_position.line, current_position.column
            );
        }

        self.next_char();

        return Token {
            token_type: TokenType::Character(c as char),
            position: current_position,
        };
    }

    fn read_string(&mut self) -> Token {
        let current_position = self.file_position.clone();

        let mut buffer: Vec<u8> = Vec::new();

        let mut c = self.next_char();

        let mut escape = false;

        while ((c == b'"' && escape) || (c != b'"')) && !self.reached_eof {
            if escape {
                match c {
                    b'"' => buffer.push(b'\"'),
                    b'n' => buffer.push(b'\n'),
                    b't' => buffer.push(b'\t'),
                    b'r' => buffer.push(b'\r'),
                    b'0' => buffer.push(b'\0'),
                    b'\\' => buffer.push(b'\\'),
                    _ => {}
                }
                escape = false;
            } else {
                if c == b'\\' {
                    escape = true;
                } else {
                    buffer.push(c);
                }
            }

            c = self.next_char();
        }

        if c != b'"' {
            panic!(
                "{}:{}:{}: Expected closing string sign",
                self.filename, current_position.line, current_position.column
            );
        }

        self.next_char();

        let label = String::from_utf8(buffer).expect("Ut8 error");

        return Token {
            token_type: TokenType::StringLiteral(label),
            position: current_position,
        };
    }

    fn read_identifier(&mut self) -> Token {
        let current_position = self.file_position.clone();

        let mut buffer: Vec<u8> = Vec::new();

        let mut c = self.current_char;

        while (c as char).is_alphanumeric() || c == b'_' && !self.reached_eof {
            buffer.push(c);
            c = self.next_char();
        }

        let label = String::from_utf8(buffer).expect("Ut8 error");

        return match label.as_str() {
            "return" => Token {
                token_type: TokenType::Return,
                position: current_position,
            },
            "if" => Token {
                token_type: TokenType::If,
                position: current_position,
            },
            "while" => Token {
                token_type: TokenType::While,
                position: current_position,
            },
            "for" => Token {
                token_type: TokenType::For,
                position: current_position,
            },
            "true" => Token {
                token_type: TokenType::True,
                position: current_position,
            },
            "false" => Token {
                token_type: TokenType::False,
                position: current_position,
            },
            _ => Token {
                token_type: TokenType::Identifier(label),
                position: current_position,
            },
        };
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
                b'0'..=b'9' => c - b'0',
                b'A'..=b'F' => 10 + c - b'A',
                b'a'..=b'f' => 10 + c - b'a',
                _ => {
                    panic!(
                        "{}:{}:{}: Invalid hexadecimal number",
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
