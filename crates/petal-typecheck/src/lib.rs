mod r#type;

use petal_ast::{Block, Definition, Expression, ExpressionId, ParseResult, Statement, TypeExpression};
use petal_diagnostic::{Diagnostic, DiagnosticSeverity};
pub use r#type::*;

struct Typechecker {
    result: TypecheckerResult,

    /// The ID of the type that the current function is expected to return.
    return_type: Option<TypeId>,
}

impl Typechecker {
    /// Create a new [`Typechecker`].
    fn new(expression_count: u32) -> Self {
        Self {
            result: TypecheckerResult::new(expression_count),
            return_type: None,
        }
    }
}

impl Typechecker {
    /// Checks the types involved in the provided definition.
    fn check_definition(&mut self, definition: &Definition) {
        match definition {
            Definition::Function(function) => {
                debug_assert!(
                    self.return_type.is_none(),
                    "tried to check a function definition, but the last one did not clean up properly"
                );

                let return_type = match &function.return_type {
                    Some(expression) => self.resolve(expression),
                    None => self.result.types.void(),
                };

                self.return_type = Some(return_type);
                self.check_block(&function.body);
                self.return_type = None;
            }
        }
    }

    /// Checks the types involved in the provided [`Block`] (recursively).
    fn check_block(&mut self, block: &Block) {
        for statement in &block.statements {
            self.check_statement(statement);
        }
    }

    /// Checks the types involved in the provided statement.
    fn check_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Return(r#return) => {
                let (value_type, span) = match r#return.value.as_ref() {
                    Some(value) => (self.check_expression(value), value.span()),
                    None => (self.result.types.void(), statement.span()),
                };

                let return_type = self
                    .return_type
                    .expect("return statement outside of function definition");

                if !self.types_match(value_type, return_type) {
                    self.result.diagnostics.push(Diagnostic::error(
                        span,
                        // TODO: TypeId -> Display
                        "returned value does not match function's return type",
                    ));
                }
            }
        }
    }

    /// Checks the types involved in the provided expression.
    fn check_expression(&mut self, expression: &Expression) -> TypeId {
        let (id, type_kind) = match expression {
            // TODO: Infer the size of the integer based on its value.
            Expression::IntegerLiteral { id, .. } => (id, TypeKind::Integer(32)),
        };

        let type_id = self.result.types.intern(type_kind);
        self.result.set_expression_type(*id, type_id);
        type_id
    }
}

impl Typechecker {
    /// Resolve the type described by a [`TypeExpression`].
    fn resolve(&mut self, expression: &TypeExpression) -> TypeId {
        let kind = match expression.name.as_str() {
            "i32" => TypeKind::Integer(32),

            _ => {
                self.result.diagnostics.push(Diagnostic::error(
                    expression.span,
                    format!("unknown type: '{}'", expression.name),
                ));

                TypeKind::Invalid
            }
        };

        self.result.types.intern(kind)
    }

    /// Checks whether the types referenced by the provided [`TypeId`]s are compatible.
    fn types_match(&self, a: TypeId, b: TypeId) -> bool {
        let invalid = self.result.types.invalid();
        a == b || a == invalid || b == invalid
    }
}

/// The result of the typechecker.
pub struct TypecheckerResult {
    /// The types used in this program.
    pub types: Types,

    /// The type of each expression, indexed by [`ExpressionId`]. Use [`TypecheckerResult::type_of`] to read it.
    expression_types: Vec<Option<TypeId>>,

    /// The diagnostics produced while checking the source.
    pub diagnostics: Vec<Diagnostic>,
}

impl TypecheckerResult {
    /// Create a new [`TypecheckerResult`] based on the provided expression count.
    fn new(expression_count: u32) -> Self {
        Self {
            types: Types::default(),
            expression_types: vec![None; expression_count as usize],
            diagnostics: Vec::new(),
        }
    }

    /// Sets the type of an expression to the provided [`TypeId`].
    fn set_expression_type(&mut self, id: ExpressionId, type_id: TypeId) {
        let slot = &mut self.expression_types[id.index()];
        debug_assert!(slot.is_none(), "expression {id:?} was checked twice");
        *slot = Some(type_id);
    }
}

impl TypecheckerResult {
    /// Return the [`TypeId`] of the expression with the provided [`ExpressionId`].
    ///
    /// # Panics
    ///
    /// This function will panic if the expression was not type checked.
    pub fn type_of(&self, id: ExpressionId) -> TypeId {
        self.expression_types[id.index()].expect("expression was not type checked")
    }

    /// Return whether compilation can continue (i.e. whether there are any error diagnostics in the result).
    pub fn can_continue(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|it| it.severity != DiagnosticSeverity::Error)
    }
}

/// Check the types of the definitions in the provided [`ParseResult`].
pub fn check(parse_result: &ParseResult) -> TypecheckerResult {
    let mut typechecker = Typechecker::new(parse_result.expression_count);
    for definition in &parse_result.definitions {
        typechecker.check_definition(definition);
    }

    typechecker.result
}

#[cfg(test)]
mod test {
    use petal_span::Span;

    use super::*;

    fn parse_and_check(source: &str) -> (ParseResult, TypecheckerResult) {
        let parse_result = petal_ast::parse(source, petal_lexer::tokenize(source));
        assert!(parse_result.can_continue(), "test source failed to parse");

        let check_result = check(&parse_result);
        (parse_result, check_result)
    }

    /// Return the value of the first `return` statement in the first function definition.
    fn first_return_value(parse_result: &ParseResult) -> &Expression {
        let Definition::Function(function) = &parse_result.definitions[0];
        let Statement::Return(r#return) = &function.body.statements[0];
        r#return.value.as_ref().expect("return statement has no value")
    }

    #[test]
    fn checks_matching_return_type() {
        let (parse_result, result) = parse_and_check("func main() -> i32 { return 1; }");

        assert!(result.can_continue());

        assert_eq!(result.diagnostics, vec![]);

        let value = first_return_value(&parse_result);
        assert_eq!(*result.types.get(result.type_of(value.id())), TypeKind::Integer(32));
    }

    #[test]
    fn checks_empty_return_in_void_function() {
        let (_, result) = parse_and_check("func main() { return; }");

        assert!(result.can_continue());

        assert_eq!(result.diagnostics, vec![]);
    }

    #[test]
    fn cannot_return_value_from_void_function() {
        let (_, result) = parse_and_check("func main() { return 1; }");

        assert!(!result.can_continue());

        assert_eq!(result.diagnostics, vec![Diagnostic::error(
            Span::new(21, 22),
            "returned value does not match function's return type"
        )]);
    }

    #[test]
    fn cannot_return_nothing_from_non_void_function() {
        let (_, result) = parse_and_check("func main() -> i32 { return; }");

        assert!(!result.can_continue());

        assert_eq!(result.diagnostics, vec![Diagnostic::error(
            Span::new(21, 27),
            "returned value does not match function's return type"
        )]);
    }

    #[test]
    fn unknown_return_type_produces_one_diagnostic() {
        let (_, result) = parse_and_check("func main() -> foo { return 1; }");

        assert!(!result.can_continue());

        assert_eq!(result.diagnostics, vec![Diagnostic::error(
            Span::new(15, 18),
            "unknown type: 'foo'"
        )]);
    }
}
