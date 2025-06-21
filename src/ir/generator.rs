use crate::{
    ast::node::{
        expression::Expression,
        statement::{FunctionDefinition, Statement},
    },
    ir::{
        Function, Operation, Value,
        context::Context,
        error::{IRError, IRErrorKind},
        expression::ExpressionVisitor,
        statement::StatementVisitor,
    },
};

/// Responsible for converting a tree of AST nodes into an IR.
pub struct IntermediateRepresentation {
    pub(crate) context: Context,
}

/// A result-type for IR generation functions.
pub type IRResult<T> = Result<T, IRError>;

impl IntermediateRepresentation {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    pub fn parse(&mut self, ast: &Vec<Statement>) -> IRResult<Vec<Function>> {
        let mut functions = vec![];

        // The intermediate representation can only compile function blocks at the top level.
        for node in ast {
            if let Statement::FunctionDefinition(definition) = &node {
                functions.push(self.parse_function_definition(definition)?);
            } else {
                return Err(IRError::new(
                    IRErrorKind::UnsupportedTopLevelStatement(node.clone()),
                    None,
                ));
            }
        }

        Ok(functions)
    }

    fn parse_function_definition(&mut self, definition: &FunctionDefinition) -> IRResult<Function> {
        let mut body = vec![];

        self.context.start_function_scope()?;

        for statement in &definition.body {
            IntermediateRepresentation::visit_statement(&mut self.context, statement, &mut body)?;
        }

        // We need to know how much space to allocate on the stack.
        let variables = self
            .context
            .function_scope(Some(definition.node.location))?
            .variables
            .clone();

        self.context.end_function_scope();

        Ok(Function {
            body,
            name: definition.name.clone(),
            variables,
        })
    }

    pub(crate) fn visit_statement(
        context: &mut Context,
        statement: &Statement,
        operations: &mut Vec<Operation>,
    ) -> IRResult<()> {
        match statement {
            Statement::VariableDeclaration(declaration) => declaration.visit(context, operations),
            Statement::VariableReassignment(reassignment) => reassignment.visit(context, operations),
            Statement::Return(r#return) => r#return.visit(context, operations),

            _ => Err(IRError::new(IRErrorKind::UnsupportedStatement(statement.clone()), None)),
        }
    }

    pub(crate) fn visit_expression(context: &mut Context, expression: &Expression) -> IRResult<Value> {
        match expression {
            Expression::IntegerLiteral(literal) => literal.visit(context),
            Expression::IdentifierReference(identifier_reference) => identifier_reference.visit(context),

            _ => Err(IRError::new(
                IRErrorKind::UnsupportedExpression(expression.clone()),
                None,
            )),
        }
    }
}
