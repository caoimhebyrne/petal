use crate::{
    ast::{
        error::ASTErrorKind,
        expression::{Expression, ExpressionKind},
        statement::{Statement, StatementKind, VariableDeclaration},
    },
    core::error::Error,
    lexer::{
        Lexer,
        token::{Keyword, TokenKind},
    },
};

pub mod error;
pub mod expression;
pub mod statement;

/// Converts tokens from a [Lexer] into an Abstract Syntax Tree.
pub struct ASTParser<'a> {
    /// The lexer to read tokens from.
    lexer: &'a mut Lexer<'a>,
}

impl<'a> ASTParser<'a> {
    /// Creates a new [ASTParser] which reads from the provided [Lexer].
    pub fn new(lexer: &'a mut Lexer<'a>) -> ASTParser<'a> {
        return ASTParser { lexer };
    }

    /// Returns the next AST node at the current position in the source code.
    pub fn next_statement(&mut self) -> Result<Statement, Error> {
        // The only statement kind that is supported is the variable declaration.
        let let_token = self.lexer.next_token()?;
        if let_token.kind != TokenKind::Keyword(Keyword::Let) {
            return Error {
                kind: ASTErrorKind::UnexpectedEndOfFile.into(),
                span: let_token.span,
            }
            .into();
        }

        let identifier_token = self.lexer.next_token()?;
        let identifier = match identifier_token.kind {
            TokenKind::Identifier(identifier) => identifier,
            _ => {
                return Error {
                    kind: ASTErrorKind::UnexpectedEndOfFile.into(),
                    span: identifier_token.span,
                }
                .into();
            }
        };

        let equals_token = self.lexer.next_token()?;
        if equals_token.kind != TokenKind::Equals {
            return Error {
                kind: ASTErrorKind::UnexpectedEndOfFile.into(),
                span: equals_token.span,
            }
            .into();
        }

        let expression = self.next_expression()?;

        let variable_declaration = VariableDeclaration {
            name: identifier,
            value: expression,
        };

        Ok(Statement {
            kind: StatementKind::VariableDeclaration(variable_declaration),
            span: let_token.span,
        })
    }

    fn next_expression(&mut self) -> Result<Expression, Error> {
        // The only expression type that is supported is the integer literal.
        let token = self.lexer.next_token()?;

        let integer_literal = match token.kind {
            TokenKind::IntegerLiteral(literal) => literal,
            _ => {
                return Error {
                    kind: ASTErrorKind::UnexpectedEndOfFile.into(),
                    span: token.span,
                }
                .into();
            }
        };

        Ok(Expression {
            kind: ExpressionKind::IntegerLiteral(integer_literal),
            span: token.span,
        })
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.\
    use super::*;
    use crate::core::source_span::SourceSpan;

    #[test]
    fn test_variable_declaration() {
        let mut lexer = Lexer::new("let identifier = 123456;");
        let mut ast_parser = ASTParser::new(&mut lexer);

        assert_eq!(
            ast_parser.next_statement().expect("next_statement should not fail!"),
            Statement {
                kind: StatementKind::VariableDeclaration(VariableDeclaration {
                    name: "identifier".to_string(),
                    value: Expression {
                        kind: ExpressionKind::IntegerLiteral(123456),
                        span: SourceSpan { start: 16, end: 23 }
                    }
                }),
                span: SourceSpan { start: 0, end: 3 }
            }
        )
    }
}
