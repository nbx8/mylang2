use crate::token::Kind;
use crate::token::Token;
use std::str;

pub struct Lexer<'a> {
    input: &'a [u8],
    position: usize,
    read_position: usize,
    byte: u8,
}
impl<'a> Lexer<'a> {
    #[must_use = "Creates a Lexer, has no side effects"]
    pub fn new(input: &'a str) -> Lexer {
        let mut lexer = Lexer {
            input: input.as_bytes(),
            position: 0,
            read_position: 0,
            byte: 0,
        };
        lexer.step();
        lexer
    }

    // Returns the current character.
    fn char(&self) -> char {
        char::from(self.byte)
    }

    // Advances the lexer.
    fn step(&mut self) {
        self.byte = match self.input.get(self.read_position) {
            None => 0,
            Some(b) => {
                self.position = self.read_position;
                self.read_position += 1;
                *b
            }
        }
    }

    // Resets the lexer to a previous position.
    fn reset(&mut self, position: usize) {
        self.position = position;
        self.read_position = position + 1;
        self.byte = match self.input.get(self.position) {
            None => 0,
            Some(b) => *b,
        }
    }

    // Returns the next character without advancing the lexer.
    fn peek_char(&self) -> char {
        match self.input.get(self.read_position) {
            None => '\0',
            Some(c) => char::from(*c),
        }
    }

    // Returns the text between the start and the read position.
    fn text_range(&self, start: usize) -> &'a [u8] {
        &self.input[start..self.read_position]
    }

    // Returns a token for a single character.
    fn char_token(&self, kind: Kind) -> Token<'a> {
        self.text_token(self.position, kind)
    }

    // Returns a token for a range of text.
    fn text_token(&self, start: usize, kind: Kind) -> Token<'a> {
        Token::new(self.text_range(start), start, kind)
    }

    // Attempts to read a whitespace token.
    fn maybe_read_whitespace(&mut self) -> Option<Token<'a>> {
        if !self.char().is_whitespace() {
            return None;
        }

        let start = self.position;
        while self.peek_char().is_whitespace() {
            self.step();
        }

        Some(self.text_token(start, Kind::Whitespace))
    }

    // Attempts to read an identifier token, potentially advancing the lexer.
    fn maybe_read_identifier(&mut self) -> Option<Token<'a>> {
        if !self.char().is_ascii_alphabetic() {
            return None;
        }

        let start = self.position;
        while self.peek_char().is_ascii_alphanumeric() || self.peek_char() == '_' {
            self.step();
        }
        Some(self.text_token(start, Kind::Identifier))
    }

    // Attempts to read a keyword token, potentially advancing the lexer.
    fn maybe_read_keyword(&mut self) -> Option<Token<'a>> {
        if !self.char().is_ascii_alphabetic() {
            return None;
        }

        let start = self.position;
        while self.peek_char().is_alphanumeric() {
            self.step();
        }
        match str::from_utf8(self.text_range(start)) {
            Ok(text) => match crate::token::KEYWORDS.get(text) {
                Some(kind) => Some(self.text_token(start, *kind)),
                _ => {
                    self.reset(start);
                    None
                }
            },
            _ => {
                self.reset(start);
                None
            }
        }
    }

    // Attempts to read a symbol token, potentially advancing the lexer.
    fn maybe_read_symbol(&mut self) -> Option<Token<'a>> {
        return if self.char() == '=' {
            Some(self.char_token(Kind::EqualSign))
        } else if self.char() == ':' {
            Some(self.char_token(Kind::Colon))
        } else if self.char() == '+' {
            Some(self.char_token(Kind::Plus))
        } else if self.char() == '-' {
            return if self.peek_char() == '>' {
                let start = self.position;
                self.step();
                Some(self.text_token(start, Kind::Arrow))
            } else {
                Some(self.char_token(Kind::Minus))
            };
        } else if self.char() == '/' {
            Some(self.char_token(Kind::Divide))
        } else if self.char() == '*' {
            Some(self.char_token(Kind::Star))
        } else if self.char() == '(' {
            Some(self.char_token(Kind::LeftParenthesis))
        } else if self.char() == ')' {
            Some(self.char_token(Kind::RightParenthesis))
        } else if self.char() == '[' {
            Some(self.char_token(Kind::LeftSquareBracket))
        } else if self.char() == ']' {
            Some(self.char_token(Kind::RightSquareBracket))
        } else if self.char() == '{' {
            Some(self.char_token(Kind::LeftBrace))
        } else if self.char() == '}' {
            Some(self.char_token(Kind::RightBrace))
        } else {
            None
        };
    }

    // Attempts to read an integer token, potentially advancing the lexer.
    fn maybe_read_integer(&mut self) -> Option<Token<'a>> {
        if !self.char().is_ascii_digit() {
            return None;
        }
        let start = self.position;
        while self.peek_char().is_ascii_digit() {
            self.step();
        }
        return Some(self.text_token(start, Kind::IntegerLiteral));
    }

    // Attempts to read a string token, potentially advancing the lexer.
    fn maybe_read_string(&mut self) -> Option<Token<'a>> {
        if self.char() != '"' {
            return None;
        }
        let start = self.position;
        self.step(); // consume the opening quote.
        while self.peek_char() != '"' {
            // Returns None if the string is incomplete.
            if self.peek_char() == '\0' {
                self.reset(start);
                return None;
            }
            self.step();
        }

        let token = self.text_token(start + 1, Kind::String);
        self.step(); // consume the closing quote.
        Some(token)
    }

    fn maybe_read_comment(&mut self) -> Option<Token<'a>> {
        if self.char() != '#' {
            return None;
        }
        let start = self.position;
        while self.peek_char() != '\n' {
            self.step();
            // Returns `None` if the comment does not end with a newline.
            if self.peek_char() == '\0' {
                self.reset(start);
                return None;
            }
        }
        let token = Some(self.text_token(start, Kind::Comment));
        self.step(); // consume the newline.
        token
    }

    // Reads the next token unconditionally advancing the lexer.
    fn read_token(&mut self) -> Token<'a> {
        if self.char() == '\0' {
            return Token::end_of_file(self.position);
        } else if let Some(t) = self.maybe_read_whitespace() {
            return t;
        } else if let Some(t) = self.maybe_read_comment() {
            return t;
        } else if let Some(t) = self.maybe_read_symbol() {
            return t;
        } else if let Some(t) = self.maybe_read_keyword() {
            return t;
        } else if let Some(t) = self.maybe_read_string() {
            return t;
        } else if let Some(t) = self.maybe_read_integer() {
            return t;
        } else if let Some(t) = self.maybe_read_identifier() {
            return t;
        } else {
            let start = self.position;
            while self.char() != '\0' {
                self.step();
            }
            return self.text_token(start, Kind::Unknown);
        }
    }

    pub fn next_token(&mut self) -> Token<'a> {
        let token = self.read_token();
        self.step();
        token
    }

    // Returns the 1-based row number of the given Token.
    pub fn get_row(&self, token: &Token) -> usize {
        assert!(token.kind() != Kind::EndOfFile);
        assert!(token.offset() <= self.input.len());
        let mut row = 1;
        for c in self.input[..token.offset()].iter() {
            if *c == b'\n' {
                row += 1;
            }
        }
        row
    }

    // Returns the 1-based column number of the given token.
    pub fn get_column(&self, token: &Token) -> usize {
        assert!(token.kind() != Kind::EndOfFile);
        assert!(token.offset() <= self.input.len());
        let mut column = 1;
        for c in self.input[..token.offset()].iter().rev() {
            if *c == b'\n' {
                break;
            }
            column += 1;
        }
        column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! lexer_test_case {
        ( name: $test_name:ident, input: $input:expr, expected_tokens:$expected_tokens:expr,) => {
            #[test]
            fn $test_name() {
                let mut lexer = Lexer::new($input);
                for expected_token in $expected_tokens {
                    let mut t = lexer.next_token();
                    while t.kind() == Kind::Whitespace {
                        t = lexer.next_token();
                    }
                    assert_eq!(t.kind(), expected_token.1);
                    assert_eq!(t.text(), expected_token.0);
                }
                assert_eq!(lexer.next_token().kind(), Kind::EndOfFile);
            }
        };
        ( name: $test_name:ident, input: $input:expr, expected_tokens:$expected_tokens:expr, check_whitespace: true,) => {
            #[test]
            fn $test_name() {
                let mut lexer = Lexer::new($input);
                for expected_token in $expected_tokens {
                    let t = lexer.next_token();
                    assert_eq!(t.kind(), expected_token.1);
                    assert_eq!(t.text(), expected_token.0);
                }
                assert_eq!(lexer.next_token().kind(), Kind::EndOfFile);
            }
        };
    }

    lexer_test_case! {
        name: single_space,
        input: " ",
        expected_tokens: &[
            (" ", Kind::Whitespace),
        ],
        check_whitespace: true,
    }

    lexer_test_case! {
        name: space_and_tab,
        input: " \t",
        expected_tokens: &[
            (" \t", Kind::Whitespace),
        ],
        check_whitespace: true,
    }

    lexer_test_case! {
        name: let_keyword,
        input: "let",
        expected_tokens: &[
            ("let", Kind::Let),
        ],
    }

    lexer_test_case! {
        name: let_statement,
        input: "let x",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer,
        input: "let x = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_mutable_integer,
        input: "let mut x = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("mut", Kind::Mut),
            ("x", Kind::Identifier),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i1,
        input: "let x: int1 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int1", Kind::Int1),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i2,
        input: "let x: int2 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int2", Kind::Int2),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i4,
        input: "let x: int4 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int4", Kind::Int4),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i8,
        input: "let x: int8 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int8", Kind::Int8),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i16,
        input: "let x: int16 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int16", Kind::Int16),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i32,
        input: "let x: int32 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int32", Kind::Int32),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_integer_i64,
        input: "let x: int64 = 5",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int64", Kind::Int64),
            ("=", Kind::EqualSign),
            ("5", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: let_statement_with_assignment_to_string,
        input: r#"let x: int64 = "five""#,
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int64", Kind::Int64),
            ("=", Kind::EqualSign),
            ("five", Kind::String),
        ],
    }

    lexer_test_case! {
        name: incomplete_string,
        input: r#""oops"#,
        expected_tokens: &[
            (r#""oops"#, Kind::Unknown),
        ],
    }

    lexer_test_case! {
        name: empty_input,
        input: "",
        expected_tokens: Vec::<(String, Kind)>::new(),
    }

    lexer_test_case! {
        name: plus,
        input: "4 + 1",
        expected_tokens: &[
            ("4", Kind::IntegerLiteral),
            ("+", Kind::Plus),
            ("1", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: minus,
        input: "4 - 1",
        expected_tokens: &[
            ("4", Kind::IntegerLiteral),
            ("-", Kind::Minus),
            ("1", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: divide,
        input: "4 / 2",
        expected_tokens: &[
            ("4", Kind::IntegerLiteral),
            ("/", Kind::Divide),
            ("2", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: multiply,
        input: "4 * 2",
        expected_tokens: &[
            ("4", Kind::IntegerLiteral),
            ("*", Kind::Star),
            ("2", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: braces_brackets_and_parens,
        input: "()[]{}",
        expected_tokens: &[
            ("(", Kind::LeftParenthesis),
            (")", Kind::RightParenthesis),
            ("[", Kind::LeftSquareBracket),
            ("]", Kind::RightSquareBracket),
            ("{", Kind::LeftBrace),
            ("}", Kind::RightBrace),
        ],
    }

    lexer_test_case! {
        name: fn_keyword_and_arrow,
        input: "fn sq(x: int32) -> int32",
        expected_tokens: &[
            ("fn", Kind::Fn),
            ("sq", Kind::Identifier),
            ("(", Kind::LeftParenthesis),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("int32", Kind::Int32),
            (")", Kind::RightParenthesis),
            ("->", Kind::Arrow),
            ("int32", Kind::Int32),
        ],
    }

    lexer_test_case! {
        name: float16,
        input: "let x:float16 = 0",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("float16", Kind::Float16),
            ("=", Kind::EqualSign),
            ("0", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: bfloat16,
        input: "let x:bfloat16 = 0",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("bfloat16", Kind::BFloat16),
            ("=", Kind::EqualSign),
            ("0", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: float32,
        input: "let x:float32 = 0",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("float32", Kind::Float32),
            ("=", Kind::EqualSign),
            ("0", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: float64,
        input: "let x:float64 = 0",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("float64", Kind::Float64),
            ("=", Kind::EqualSign),
            ("0", Kind::IntegerLiteral),
        ],
    }

    lexer_test_case! {
        name: comment,
        input: "let x:float64 = 0 # this is a comment\n",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("float64", Kind::Float64),
            ("=", Kind::EqualSign),
            ("0", Kind::IntegerLiteral),
            ("# this is a comment", Kind::Comment),
        ],
    }

    lexer_test_case! {
        name: comment_no_newline,
        input: "let x:float64 = 0 # this is not a comment",
        expected_tokens: &[
            ("let", Kind::Let),
            ("x", Kind::Identifier),
            (":", Kind::Colon),
            ("float64", Kind::Float64),
            ("=", Kind::EqualSign),
            ("0", Kind::IntegerLiteral),
            ("# this is not a comment", Kind::Unknown),
        ],
    }

    #[test]
    fn test_row_and_column() {
        let input_source = "\
let x = 5
let y = 7

let z = x + y
";
        let mut lexer = Lexer::new(input_source);
        let mut tokens = Vec::<Token>::new();
        let mut t = lexer.next_token();
        while t.kind() != Kind::EndOfFile {
            tokens.push(t);
            t = lexer.next_token();
        }

        let expected_tokens = &[
            // (text, kind, row, column)
            // Line 1
            ("let", Kind::Let, 1, 1),
            (" ", Kind::Whitespace, 1, 4),
            ("x", Kind::Identifier, 1, 5),
            (" ", Kind::Whitespace, 1, 6),
            ("=", Kind::EqualSign, 1, 7),
            (" ", Kind::Whitespace, 1, 8),
            ("5", Kind::IntegerLiteral, 1, 9),
            ("\n", Kind::Whitespace, 1, 10),
            // Line 2
            ("let", Kind::Let, 2, 1),
            (" ", Kind::Whitespace, 2, 4),
            ("y", Kind::Identifier, 2, 5),
            (" ", Kind::Whitespace, 2, 6),
            ("=", Kind::EqualSign, 2, 7),
            (" ", Kind::Whitespace, 2, 8),
            ("7", Kind::IntegerLiteral, 2, 9),
            ("\n\n", Kind::Whitespace, 2, 10),
            // Line 4
            // Line 3
            ("let", Kind::Let, 4, 1),
            (" ", Kind::Whitespace, 4, 4),
            ("z", Kind::Identifier, 4, 5),
            (" ", Kind::Whitespace, 4, 6),
            ("=", Kind::EqualSign, 4, 7),
            (" ", Kind::Whitespace, 4, 8),
            ("x", Kind::Identifier, 4, 9),
            (" ", Kind::Whitespace, 4, 10),
            ("+", Kind::Plus, 4, 11),
            (" ", Kind::Whitespace, 4, 12),
            ("y", Kind::Identifier, 4, 13),
            ("\n", Kind::Whitespace, 4, 14),
        ];
        for (i, expected_token) in expected_tokens.iter().enumerate() {
            assert_eq!(tokens[i].text(), expected_token.0);
            assert_eq!(tokens[i].kind(), expected_token.1);
            assert_eq!(lexer.get_row(&tokens[i]), expected_token.2);
            assert_eq!(lexer.get_column(&tokens[i]), expected_token.3);
        }
    }
}
