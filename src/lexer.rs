#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),

    // Keywords
    Local,
    Function,
    End,
    If,
    Then,
    Else,
    While,
    Do,
    For,
    In,
    Return,
    Break,
    True,
    False,
    Nil,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    Assign,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Dot,
    DotDot, // .. concatenation operator

    // Special
    Eof,
    Newline,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.advance();

        let token_type = match ch {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Percent,
            '^' => TokenType::Caret,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            ',' => TokenType::Comma,
            ';' => TokenType::Semicolon,
            '.' => {
                if self.peek() == '.' {
                    self.advance(); // consume the second '.'
                    TokenType::DotDot
                } else {
                    TokenType::Dot
                }
            }
            '\n' => TokenType::Newline,

            '=' => {
                if self.match_char('=') {
                    TokenType::Equal
                } else {
                    TokenType::Assign
                }
            }

            '~' => {
                if self.match_char('=') {
                    TokenType::NotEqual
                } else {
                    return Err(format!("Unexpected character '~' at {}:{}", start_line, start_column));
                }
            }

            '<' => {
                if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }

            '>' => {
                if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }

            '"' | '\'' => self.string_literal(ch)?,

            ch if ch.is_ascii_digit() => self.number_literal()?,

            ch if ch.is_alphabetic() || ch == '_' => self.identifier_or_keyword()?,

            _ => return Err(format!("Unexpected character '{}' at {}:{}", ch, start_line, start_column)),
        };

        Ok(Token {
            token_type,
            line: start_line,
            column: start_column,
        })
    }

    fn string_literal(&mut self, quote: char) -> Result<TokenType, String> {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != quote {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            value.push(self.advance());
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        self.advance(); // Closing quote
        Ok(TokenType::String(value))
    }

    fn number_literal(&mut self) -> Result<TokenType, String> {
        let start = self.position - 1;

        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            self.advance();
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        let number = number_str.parse::<f64>()
            .map_err(|_| format!("Invalid number: {}", number_str))?;

        Ok(TokenType::Number(number))
    }

    fn identifier_or_keyword(&mut self) -> Result<TokenType, String> {
        let start = self.position - 1;

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text: String = self.input[start..self.position].iter().collect();

        let token_type = match text.as_str() {
            "local" => TokenType::Local,
            "function" => TokenType::Function,
            "end" => TokenType::End,
            "if" => TokenType::If,
            "then" => TokenType::Then,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "do" => TokenType::Do,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "nil" => TokenType::Nil,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Identifier(text),
        };

        Ok(token_type)
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '-' if self.peek_next() == Some('-') => {
                    // Skip comment
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.input[self.position];
        self.position += 1;
        self.column += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input.get(self.position).copied().unwrap_or('\0')
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.input[self.position] != expected {
            false
        } else {
            self.position += 1;
            self.column += 1;
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
