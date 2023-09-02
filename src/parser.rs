#![allow(irrefutable_let_patterns)]

use crate::{
    ast::Program,
    ast::{
        self, BinaryExpression, Expression, Indentifier, IntegerLiteral, LetStatement, Statement,
        Type,
    },
    token::{Kind, Token},
};

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
}

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    position: usize,
    read_position: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Parser<'a> {
        let mut parser = Parser {
            tokens,
            position: 0,
            read_position: 0,
        };
        assert!(!parser.tokens.is_empty());
        assert!(parser.tokens.last().unwrap().kind() == Kind::EndOfFile);
        parser.step();
        parser
    }

    // Returns the current token.
    fn token(&self) -> &'a Token<'a> {
        &self.tokens[self.position]
    }

    // Advances the parser.
    fn step(&mut self) {
        if self.read_position >= self.tokens.len() {
            return;
        }
        self.position = self.read_position;
        self.read_position += 1;
        // Ignore whitespace.
        while self.token().kind() == Kind::Whitespace && self.read_position < self.tokens.len() {
            self.position = self.read_position;
            self.read_position += 1;
        }
    }

    // Resets the parser to a previous position.
    fn reset(&mut self, position: usize) {
        self.position = position;
        self.read_position = position + 1;
    }

    fn try_parse_let_stmt(&mut self) -> Result<Statement<'a>, String> {
        assert!(self.token().kind() == Kind::Let);
        let start = self.position;
        self.step(); // Consume the "let" token.

        // See if we have a `mut` keyword
        let mut_token = self.token();
        let mutable = match mut_token.kind() {
            Kind::Mut => {
                self.step(); // Consume the "mut" token.
                true
            }
            _ => false,
        };

        let identifier_token = self.token();
        let identifier = match identifier_token.kind() {
            Kind::Identifier => Indentifier {
                name: identifier_token.text(),
            },
            _ => {
                self.reset(start);
                return Err(format!("Expected identifier, got {:?}", identifier_token));
            }
        };
        self.step(); // Consume the identifier.

        let colon = self.token();
        if colon.kind() != Kind::Colon {
            self.reset(start);
            return Err(format!("Expected colon, got {:?}", colon));
        }
        self.step(); // Consume the colon.

        let ttype_token = self.token();
        let ttype = match ttype_token.kind() {
            Kind::Int32 => ast::Type { name: "int32" },
            _ => {
                self.reset(start);
                return Err(format!("Expected type, got {:?}", colon));
            }
        };
        self.step(); // Consume the type.

        let equals_token = self.token();
        match equals_token.kind() {
            Kind::EqualSign => (),
            _ => {
                self.reset(start);
                return Err(format!("Expected equals, got {:?}", equals_token));
            }
        }
        self.step(); // Consume the equals symbol.

        let expression_token = self.token();
        let expression = match expression_token.kind() {
            Kind::IntegerLiteral => crate::ast::Expression::IntegerLiteral(IntegerLiteral {
                text: expression_token.text(),
            }),
            Kind::Identifier => crate::ast::Expression::Identifier(Indentifier {
                name: expression_token.text(),
            }),
            _ => {
                self.reset(start);
                return Err(format!(
                    "Expected integer literal or identifier, got {:?}",
                    expression_token
                ));
            }
        };
        self.step(); // Consume the value.

        if self.token().kind() != Kind::Semicolon {
            self.reset(start);
            return Err(format!(
                "Expected semicolon at end of statement, got {:?}",
                self.token()
            ));
        }
        self.step(); // Consume the semicolon.

        Ok(ast::Statement::Let(LetStatement {
            identifier,
            mutable,
            ttype,
            expression: Box::new(expression),
        }))
    }

    fn try_parse_binary_expression(&mut self) -> Result<Statement<'a>, String> {
        let start = self.position;
        let left_token = self.token();
        let left = match left_token.kind() {
            Kind::Identifier => {
                let id = Indentifier {
                    name: left_token.text(),
                };
                Box::new(Expression::Identifier(id))
            }
            Kind::IntegerLiteral => {
                let literal = IntegerLiteral {
                    text: left_token.text(),
                };
                Box::new(Expression::IntegerLiteral(literal))
            }
            _ => {
                self.reset(start);
                return Err(format!("Expected identifier, got {:?}", left_token));
            }
        };
        self.step(); // Consume the identifier.

        let op_token = self.token();
        let operator = match op_token.kind() {
            Kind::Plus => ast::BinaryOperator::Plus,
            Kind::Minus => ast::BinaryOperator::Minus,
            Kind::Star => ast::BinaryOperator::Star,
            Kind::Divide => ast::BinaryOperator::Divide,
            _ => {
                self.reset(start);
                return Err(format!("Expected '+', got {:?}", op_token));
            }
        };
        self.step(); // Consume the op symbol.

        let right_token = self.token();
        let right = match right_token.kind() {
            Kind::Identifier => {
                let id = Indentifier {
                    name: right_token.text(),
                };
                Box::new(Expression::Identifier(id))
            }
            Kind::IntegerLiteral => {
                let literal = IntegerLiteral {
                    text: right_token.text(),
                };
                Box::new(Expression::IntegerLiteral(literal))
            }
            _ => {
                self.reset(start);
                return Err(format!("Expected identifier, got {:?}", right_token));
            }
        };
        self.step(); // Consume the identifier.

        if self.token().kind() != Kind::Semicolon {
            self.reset(start);
            return Err(format!(
                "Expected semicolon at end of binary expression, got {:?}",
                self.token()
            ));
        }
        self.step(); // Consume the semicolon.

        let expression = Expression::BinaryExpression(BinaryExpression {
            operator,
            left,
            right,
        });
        Ok(ast::Statement::Expression(expression))
    }

    fn try_parse_function(&mut self) -> Result<Statement<'a>, String> {
        let start = self.position;
        assert!(self.token().kind() == Kind::Fn);
        self.step(); // Consume the "fn" token.

        let identifier_token = self.token();
        let identifier = match identifier_token.kind() {
            Kind::Identifier => Indentifier {
                name: identifier_token.text(),
            },
            _ => {
                self.reset(start);
                return Err(format!("Expected identifier, got {:?}", identifier_token));
            }
        };
        self.step(); // Consume the identifier.

        if self.token().kind() != Kind::LeftParenthesis {
            self.reset(start);
            return Err(format!("Expected '(', got {:?}", self.token()));
        }
        self.step(); // Consume the '(' token.

        if self.token().kind() != Kind::RightParenthesis {
            self.reset(start);
            return Err(format!("Expected ')', got {:?}", self.token()));
        }
        self.step(); // Consume the ')' token.

        if self.token().kind() != Kind::Arrow {
            self.reset(start);
            return Err(format!("Expected '->', got {:?}", self.token()));
        }
        self.step(); // Consume the '->' token.

        let return_type = match self.token().kind() {
            Kind::Int32 => Type { name: "int32" },
            _ => {
                self.reset(start);
                return Err(format!("Expected 'int32', got {:?}", self.token()));
            }
        };
        self.step(); // Consume the return type.

        if self.token().kind() != Kind::Semicolon {
            self.reset(start);
            return Err(format!(
                "Expected semicolon at end of binary expression, got {:?}",
                self.token()
            ));
        }
        self.step(); // Consume the semicolon.

        return Ok(ast::Statement::FunctionDeclaration(
            ast::FunctionDeclaration {
                identifier,
                parameters: vec![],
                ttype: return_type,
            },
        ));
    }

    // Reads the next statement.
    fn read_statement(&mut self) -> Result<Statement<'a>, String> {
        let token = self.token();
        match token.kind() {
            Kind::Let => self.try_parse_let_stmt(),
            Kind::Identifier => self.try_parse_binary_expression(),
            Kind::IntegerLiteral => self.try_parse_binary_expression(),
            Kind::Fn => self.try_parse_function(),
            _ => Err(format!("Failed to parse token {:?}", token)),
        }
    }

    // Reads the next statement and advances the parser.
    //
    // Returns an error if the statement cannot be parsed.
    // The parser is not advanced if an error is returned.
    fn next_stmt(&mut self) -> Result<Statement<'a>, String> {
        let stmt = self.read_statement();
        if stmt.is_ok() {
            self.step();
        }
        stmt
    }

    // Parses a program from tokens.
    //
    // Returns an error if the program cannot be parsed.
    pub fn parse_program(tokens: &'a [Token]) -> Result<Program<'a>, ParserError> {
        let mut parser = Parser::new(tokens);
        let mut statements = vec![];
        while parser.token().kind() != Kind::EndOfFile {
            match parser.next_stmt() {
                Ok(stmt) => statements.push(stmt),
                Err(message) => return Err(ParserError { message }),
            }
            parser.step();
        }
        Ok(Program { statements })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast, lexer::Lexer, matcher::*, parser::Parser, *};

    #[test]
    fn empty_file_can_be_parsed() {
        let input = "";
        let tokens = Lexer::tokenize(input);
        let program = Parser::parse_program(&tokens);
        assert!(program.is_ok());
        assert!(program.unwrap().statements.is_empty());
    }

    #[test]
    fn fail_to_parse_let_statement_with_no_trailing_semicolon() {
        let input = "let x: int32 = 5";
        let tokens = Lexer::tokenize(input);
        let program = Parser::parse_program(&tokens);
        match program {
            Ok(_) => {
                panic!("Expected parse error");
            }
            Err(err) => {
                assert!(err
                    .message
                    .starts_with("Expected semicolon at end of statement"));
            }
        }
    }

    #[test]
    fn test_matcher() {
        let input = "x + y;";
        let tokens = Lexer::tokenize(input);
        match Parser::parse_program(&tokens) {
            Ok(program) => {
                let matcher = match_binary_expression!();
                if let ast::Statement::Expression(expr) = &program.statements[0] {
                    assert!(matcher.matches(expr));
                } else {
                    panic!("Expected an expression statement");
                }
            }
            Err(err) => panic!("Failed to parse program: {}", err.message),
        }
    }

    macro_rules! parse_expression_test {
        (name:$name:ident, input:$input:expr, matcher:$matcher:expr) => {
            #[test]
            fn $name() {
                let tokens = Lexer::tokenize($input);
                match Parser::parse_program(&tokens) {
                    Ok(program) => {
                        if let ast::Statement::Expression(expr) = &program.statements[0] {
                            assert!($matcher.matches(expr));
                        } else {
                            panic!("Expected an expression statement");
                        }
                    }
                    Err(err) => panic!("Failed to parse program: {}", err.message),
                }
            }
        };
    }

    parse_expression_test!(name:parse_binary_plus_expression_with_identifiers,
                 input:"x + y;",
                 matcher:match_binary_expression!(
                    match_identifier!("x"),
                    ast::BinaryOperator::Plus,
                    match_identifier!("y")));

    parse_expression_test!(name:parse_binary_plus_expression_with_integer_literals,
                 input:"2 + 4;",
                 matcher:match_binary_expression!(
                    match_integer_literal!("2"),
                    ast::BinaryOperator::Plus,
                    match_integer_literal!("4")));

    parse_expression_test!(name:parse_binary_minus_expression,
        input:"2 - 4;",
        matcher:match_binary_expression!(
            match_any_expression!(),
            ast::BinaryOperator::Minus,
            match_any_expression!()));

    parse_expression_test!(name:parse_binary_star_expression,
                input:"2 * 4;",
                matcher:match_binary_expression!(
                    match_any_expression!(),
                    ast::BinaryOperator::Star,
                    match_any_expression!()));

    parse_expression_test!(name:parse_binary_divide_expression,
                        input:"2 / 4;",
                        matcher:match_binary_expression!(
                            match_any_expression!(),
                            ast::BinaryOperator::Divide,
                            match_any_expression!()));

    macro_rules! parse_statement_test {
        (name:$name:ident, input:$input:expr, matcher:$matcher:expr) => {
            #[test]
            fn $name() {
                let tokens = Lexer::tokenize($input);
                match Parser::parse_program(&tokens) {
                    Ok(program) => assert!($matcher.matches(&program.statements[0])),
                    Err(err) => panic!("Failed to parse program: {}", err.message),
                }
            }
        };
    }

    parse_statement_test! {
        name:parse_let_statement_with_integer_literal,
        input:"let x: int32 = 5;",
        matcher:match_let_statement!(
            "x",
            match_type!(),
            match_integer_literal!("5"))
    }

    parse_statement_test! {
        name:parse_let_statement_with_identifier,
        input:"let x: int32 = y;",
        matcher:match_let_statement!(
            "x",
            match_type!(),
            match_identifier!("y"))
    }

    parse_statement_test! {
        name:parse_mutable_let_statement,
        input:"let mut x: int32 = y;",
        matcher:match_mutable_let_statement!(
            "x",
            match_type!(),
            match_any_expression!())
    }

    parse_statement_test! {
        name:parse_function,
        input:"fn five() -> int32;",
        matcher:match_function_declaration!(
            "five",
            match_type!("int32"))
    }
}
