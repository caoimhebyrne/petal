use crate::ast::node::{
    expression::{Expression, IntegerLiteral},
    statement::{FunctionDefinition, Statement, VariableDeclaration},
};

/// A value which can be an argument of an [Operation] in the IR.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Value {
    IntegerLiteral(u64),
}

/// An operation in the IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Stores a value into a variable allocated on the stack.
    Store { variable_index: usize, value: Value },

    /// Allocates a variable on the stack.
    Allocate { variable_index: usize },
}

/// A function defined in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    /// The body of the function (a.k.a, the list of operations).
    pub body: Vec<Operation>,

    /// The name of the function
    pub name: String,
}

/// Responsible for converting a tree of AST nodes into an IR.
pub struct IntermediateRepresentation {
    pub context: IRContext,
}

/// The context of the intermediate representation generator.
pub struct IRContext {
    pub variables: Vec<String>,
}

impl IRContext {
    pub fn new() -> Self {
        Self { variables: Vec::new() }
    }

    pub fn declare_variable<'a>(&mut self, name: &'a str) -> usize {
        self.variables.push(name.to_string());
        return self.variables.len() - 1;
    }
}

impl IntermediateRepresentation {
    pub fn new() -> Self {
        Self {
            context: IRContext::new(),
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

        for statement in &definition.body {
            IntermediateRepresentation::visit_statement(&mut self.context, statement, &mut body);
        }

        Function {
            body,
            name: definition.name.clone(),
        }
    }

    fn visit_statement(context: &mut IRContext, statement: &Statement, operations: &mut Vec<Operation>) {
        match statement {
            Statement::VariableDeclaration(declaration) => declaration.visit_statement(context, operations),
            _ => panic!("Unable to visit statement: {:?}", statement),
        }
    }

    fn visit_expression(context: &mut IRContext, expression: &Expression) -> Value {
        match expression {
            Expression::IntegerLiteral(literal) => literal.visit(context),
            _ => panic!("Unable to visit expression: {:?}", expression),
        }
    }
}

/// Visits a statement in the AST, converting it to one or more IR operations.
trait StatementVisitor {
    fn visit_statement(&self, context: &mut IRContext, operations: &mut Vec<Operation>);
}

impl StatementVisitor for VariableDeclaration {
    fn visit_statement(&self, context: &mut IRContext, operations: &mut Vec<Operation>) {
        // A variable declaration just needs us to allocate a space on the stack.
        let index = context.declare_variable(&self.name);
        operations.push(Operation::Allocate { variable_index: index });

        // If the variable is being initialized with a value, we need to store that value
        // into the space on the stack.
        let value = IntermediateRepresentation::visit_expression(context, &self.value);

        operations.push(Operation::Store {
            variable_index: index,
            value,
        });
    }
}

/// Visits an expression inthe AST, converting it to a [Value].
trait ExpressionVisitor {
    fn visit(&self, context: &mut IRContext) -> Value;
}

impl ExpressionVisitor for IntegerLiteral {
    fn visit(&self, _context: &mut IRContext) -> Value {
        Value::IntegerLiteral(self.value)
    }
}
