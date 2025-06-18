use crate::{
    ast::node::{
        expression::Expression,
        statement::{FunctionDefinition, Statement},
    },
    ir::{Function, Operation, Value, context::Context, expression::ExpressionVisitor, statement::StatementVisitor},
};

/// Responsible for converting a tree of AST nodes into an IR.
pub struct IntermediateRepresentation {
    pub(crate) context: Context,
}

impl IntermediateRepresentation {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    pub fn parse(&mut self, ast: &Vec<Statement>) -> Vec<Function> {
        let mut functions = vec![];

        // The intermediate representation can only compile function blocks at the top level.
        for node in ast {
            if let Statement::FunctionDefinition(definition) = &node {
                functions.push(self.parse_function_definition(definition));
            } else {
                panic!(
                    "Intermediate Representation cannot handle top-level statements that are not function definitions!"
                )
            }
        }

        functions
    }

    fn parse_function_definition(&mut self, definition: &FunctionDefinition) -> Function {
        let mut body = vec![];

        self.context.start_function_scope();

        for statement in &definition.body {
            IntermediateRepresentation::visit_statement(&mut self.context, statement, &mut body);
        }

        // We need to know how much space to allocate on the stack.
        let stack_size = self.context.function_scope().variables.values().sum();

        self.context.end_function_scope();

        Function {
            body,
            name: definition.name.clone(),
            stack_size,
        }
    }

    pub(crate) fn visit_statement(context: &mut Context, statement: &Statement, operations: &mut Vec<Operation>) {
        match statement {
            Statement::VariableDeclaration(declaration) => declaration.visit(context, operations),
            Statement::Return(r#return) => r#return.visit(context, operations),

            _ => println!("Unable to visit statement: {:?}", statement),
        }
    }

    pub(crate) fn visit_expression(context: &mut Context, expression: &Expression) -> Value {
        match expression {
            Expression::IntegerLiteral(literal) => literal.visit(context),
            Expression::IdentifierReference(identifier_reference) => identifier_reference.visit(context),

            _ => panic!("Unable to visit expression: {:?}", expression),
        }
    }
}
