use std::{
    self,
    cmp::max,
    ops::Deref,
};

use crate::{
    ast::{
        expression::{
            Expression,
            ExpressionKind,
            binary_operation::{
                BinaryOperation,
                BinaryOperator,
            },
            function_call::{
                FunctionCall,
                FunctionCallArgument,
            },
            member_access::MemberAccess,
            optional_wrap::{
                OptionalEmpty,
                OptionalForceUnwrap,
                OptionalHasValue,
                OptionalWrap,
            },
            structure_initialization::{
                StructureInitialization,
                StructureInitializationField,
            },
        },
        statement::{
            Statement,
            StatementKind,
            function_declaration::{
                FunctionDeclaration,
                FunctionParameter,
            },
            r#if::If,
            namespace_declaration::NamespaceDeclaration,
            r#return::Return,
            variable_assignment::VariableAssignment,
            variable_declaration::VariableDeclaration,
        },
    },
    core::span::Span,
    module::ParsedModule,
    typechecker::{
        Typechecker,
        context::FunctionLookupRequest,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

pub(crate) struct BodyPass<'a> {
    typechecker: &'a mut Typechecker,
}

impl<'a> BodyPass<'a> {
    /// Creates a new [`BodyPass`] with the given [`Typechecker]`.
    pub fn new(typechecker: &'a mut Typechecker) -> Self {
        Self { typechecker }
    }

    /// Runs the body pass on the provided [`ParsedModule`].
    pub fn run(&mut self, modules: &mut Vec<ParsedModule>) -> Result<(), TypecheckerError> {
        for module in modules {
            for statement in &mut module.ast {
                self.visit_statement(statement)?;
            }
        }

        Ok(())
    }

    fn visit_statement(&mut self, statement: &mut Statement) -> Result<(), TypecheckerError> {
        match &mut statement.kind {
            StatementKind::FunctionDeclaration(function_declaration) => {
                self.visit_function_declaration(function_declaration, statement.span)
            }

            StatementKind::VariableDeclaration(variable_declaration) => {
                self.visit_variable_declaration(variable_declaration, statement.span)
            }

            StatementKind::Return(r#return) => self.visit_return(r#return, statement.span),

            StatementKind::VariableAssignment(variable_assignment) => {
                self.visit_variable_assignment(variable_assignment, statement.span)
            }

            StatementKind::If(r#if) => self.visit_if(r#if, statement.span),

            // We don't have to do anything at this pass for imports.
            StatementKind::Import(_) => Ok(()),
            StatementKind::TypeDeclaration(_) => Ok(()),

            StatementKind::FunctionCall(function_call) => {
                self.visit_function_call(function_call, statement.span)?;
                Ok(())
            }

            StatementKind::NamespaceDeclaration(namespace_declaration) => {
                self.visit_namespace_declaration(namespace_declaration, statement.span)
            }
        }
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`FunctionDeclaration`].
    fn visit_function_declaration(
        &mut self,
        function_declaration: &mut FunctionDeclaration,
        _span: Span,
    ) -> Result<(), TypecheckerError> {
        self.typechecker.context.push_child_scope(function_declaration.return_type.clone());

        for parameter in &function_declaration.parameters {
            self.typechecker.context.insert_variable(
                parameter.name.clone(),
                parameter.r#type.clone(),
                parameter.span,
            )?;
        }

        for statement in &mut function_declaration.body {
            self.visit_statement(statement)?;
        }

        self.typechecker.context.pop_child_scope();

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`VariableDeclaration`].
    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &mut VariableDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        // The type of the variable must be resolved.
        let variable_type = self.typechecker.resolve_type_from_expr(&variable_declaration.type_expr, span)?;

        // If the variable type is non-optional, but the expression is of the `OptionalEmpty` kind, then the variable
        // does not have an appropriate default value.
        if !matches!(variable_type, Type::Optional(_))
            && matches!(variable_declaration.value.kind, ExpressionKind::OptionalEmpty(_))
        {
            return Err(TypecheckerErrorKind::VariableDeclarationMissingInitialValue.at(span));
        }

        // The initial value for the variable must have a valid type too, and then that type must be equal to the
        // variable type.
        let value_type = self.visit_expression(&mut variable_declaration.value, Some(&variable_type))?;
        if variable_type != value_type {
            return Err(TypecheckerErrorKind::IncompatibleVariableDeclarationTypes {
                declared: variable_type,
                value: value_type,
            }
            .at(span));
        }

        variable_declaration.r#type = variable_type;
        self.typechecker.context.insert_variable_from_declaration(variable_declaration, span)?;

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`VariableAssignment`].
    fn visit_variable_assignment(
        &mut self,
        variable_assignment: &mut VariableAssignment,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        let target_type = self.visit_expression(&mut variable_assignment.target, None)?;

        // The initial value for the variable must have a valid type too, and then that type must be equal to the
        // variable type.
        let value_type = self.visit_expression(&mut variable_assignment.value, Some(&target_type))?;

        if target_type != value_type {
            return Err(TypecheckerErrorKind::IncompatibleVariableDeclarationTypes {
                declared: target_type,
                value: value_type,
            }
            .at(span));
        }

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`Return`].
    fn visit_return(&mut self, r#return: &mut Return, span: Span) -> Result<(), TypecheckerError> {
        let result_type = self.typechecker.context.scope.result_type.clone();

        let value_type = r#return
            .value
            .as_mut()
            .map(|it| self.visit_expression(it, Some(&result_type.clone())))
            .transpose()?
            .unwrap_or(Type::Void);

        // The value being returned must have the same return type as the function being parsed.
        if result_type != value_type {
            return Err(
                TypecheckerErrorKind::IncompatibleReturnTypes { declared: result_type, value: value_type }.at(span)
            );
        }

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`If`].
    fn visit_if(&mut self, r#if: &mut If, _span: Span) -> Result<(), TypecheckerError> {
        // The type of the condition must be a boolean.
        let condition_type = self.visit_expression(&mut r#if.condition, Some(&Type::Boolean))?;
        if condition_type != Type::Boolean {
            return Err(TypecheckerErrorKind::IncompatibleTypes { expected: Type::Boolean, got: condition_type }
                .at(r#if.condition.span));
        }

        // All of the statements within the block must be valid.
        self.typechecker.context.push_child_scope(self.typechecker.context.scope.result_type.clone());
        for statement in &mut r#if.block {
            self.visit_statement(statement)?;
        }
        self.typechecker.context.pop_child_scope();

        Ok(())
    }

    /// Checks and resolves the type of the provided [`Expression`].
    ///
    /// [type_hint] is the "recommended" type for the expression. In the case of something like a number literal, this
    /// is often the type of the variable that it is being assigned to, or the return type of the function.
    fn visit_expression(
        &mut self,
        expression: &mut Expression,
        type_hint: Option<&Type>,
    ) -> Result<Type, TypecheckerError> {
        let r#type = match &mut expression.kind {
            ExpressionKind::NumberLiteral(value) => BodyPass::visit_number_literal(value, type_hint, expression.span),

            ExpressionKind::StringLiteral(value) => self.visit_string_literal(value, expression.span),

            ExpressionKind::BinaryOperation(binary_operation) => {
                self.visit_binary_operation(binary_operation, type_hint, expression.span)
            }

            ExpressionKind::BooleanLiteral(value) => BodyPass::visit_boolean_literal(value, expression.span),

            ExpressionKind::FunctionCall(function_call) => {
                // If the target is a member access for an optional has_value call, then we must re-write the expression.
                if let ExpressionKind::MemberAccess(member_access) = &mut function_call.callee.kind
                    && member_access.name == "has_value"
                {
                    // The type of the target must be an optional type.
                    if let Type::Optional(_) = self.visit_expression(&mut member_access.target, None)? {
                        *expression = Expression::new(
                            ExpressionKind::OptionalHasValue(OptionalHasValue::new(*member_access.target.clone())),
                            expression.span,
                        );

                        return self.visit_expression(expression, None);
                    }
                }

                self.visit_function_call(function_call, expression.span)
            }

            ExpressionKind::IdentifierReference(name) => self.visit_identifier_reference(name, expression.span),

            ExpressionKind::Reference(inner) => self.visit_reference(inner, expression.span),

            ExpressionKind::Dereference(inner) => self.visit_dereference(inner, expression.span),

            ExpressionKind::StructureInitialization(structure_initialization) => {
                self.visit_structure_initialization(structure_initialization, type_hint, expression.span)
            }

            ExpressionKind::MemberAccess(member_access) => self.visit_member_access(member_access, expression.span),

            ExpressionKind::OptionalWrap(optional_wrap) => {
                self.visit_optional_wrap(optional_wrap, type_hint, expression.span)
            }

            ExpressionKind::OptionalEmpty(optional_empty) => {
                self.visit_optional_empty(optional_empty, type_hint, expression.span)
            }

            ExpressionKind::OptionalHasValue(optional_has_value) => {
                self.visit_optional_has_value(optional_has_value, expression.span)
            }

            ExpressionKind::OptionalForceUnwrap(optional_force_unwrap) => {
                self.visit_optional_force_unwrap(optional_force_unwrap, expression.span)
            }

            ExpressionKind::NamespaceQualifier(_) => todo!(),
        }?;

        // If the target type is an optional, and the value is not directly an optional, then we should attempt
        // to wrap it in one.
        if matches!(type_hint, Some(Type::Optional(_))) && !matches!(r#type, Type::Optional(_)) {
            // Visiting the optional wrap will re-visit the inner type, which may infer it as something else.
            //
            // e.g. if the target_type is `&i32`, and the value is a number literal, then this step will coerce
            //      the literal into `i32`.
            *expression = Expression::new(OptionalWrap::new(expression.clone()).into(), expression.span).into();
            return self.visit_expression(expression, type_hint);
        }

        Ok(r#type)
    }

    /// Checks and resolves the type of the provided number literal.
    fn visit_number_literal(value: &f64, type_hint: Option<&Type>, _span: Span) -> Result<Type, TypecheckerError> {
        // TODO: Support for floating point values.

        let minimum_integer_type = if *value < 0.0 {
            let bits = match *value {
                v if v >= i8::MIN as f64 => 8,
                v if v >= i16::MIN as f64 => 16,
                v if v >= i32::MIN as f64 => 32,
                _ => 64,
            };

            Type::SignedInteger(bits)
        } else {
            let bits = match *value {
                v if v <= u8::MAX as f64 => 8,
                v if v <= u16::MAX as f64 => 16,
                v if v <= u32::MAX as f64 => 32,
                _ => 64,
            };

            Type::UnsignedInteger(bits)
        };

        // If the type that was suggested is some integer type, then we can become that type if we are
        // compatible with it.
        let r#type = match type_hint {
            // The hint suggests that we should use a signed type. All integer types are castable to signed.
            Some(Type::SignedInteger(hint_bits)) => match minimum_integer_type {
                Type::SignedInteger(recommended_bits) => Type::SignedInteger(max(*hint_bits, recommended_bits)),
                Type::UnsignedInteger(recommended_bits) => Type::SignedInteger(max(*hint_bits, recommended_bits)),
                _ => unreachable!("recommended_integer_type can only be Type::UnsignedInteger or Type::SignedInteger"),
            },

            // The hint suggests that we should use an unsigned type. Not all integer types are castable to unsigned.
            Some(Type::UnsignedInteger(hint_bits)) => match minimum_integer_type {
                Type::UnsignedInteger(recommended_bits) => Type::UnsignedInteger(max(*hint_bits, recommended_bits)),

                // We still return the signed integer in this case, even though the hint suggests an unsigned integer.
                // The caller is responsible for checking whether the signed-ness is OK.
                Type::SignedInteger(_) => minimum_integer_type.clone(),

                _ => unreachable!("recommended_integer_type can only be Type::UnsignedInteger or Type::SignedInteger"),
            },

            // Return the recommended type, the type that was suggested is not an integer type.
            _ => minimum_integer_type.clone(),
        };

        if let Some(hint) = type_hint {
            trace!(
                "Number literal '{}' has a minimum integer type of '{}'. Type was hinted as '{}'. Final type is '{}'",
                value, minimum_integer_type, hint, r#type
            );
        }

        Ok(r#type)
    }

    /// Checks and resolves the type of the provided boolean literal.
    fn visit_boolean_literal(_value: &bool, _span: Span) -> Result<Type, TypecheckerError> {
        Ok(Type::Boolean)
    }

    /// Checks and resolves the type of the provided string literal.
    fn visit_string_literal(&self, _value: &str, _span: Span) -> Result<Type, TypecheckerError> {
        Ok(Type::Structure(self.typechecker.context.builtin_types.compile_time_str.unwrap()))
    }

    /// Checks and resolves the type of the provided [`BinaryOperation`].
    fn visit_binary_operation(
        &mut self,
        binary_operation: &mut BinaryOperation,
        type_hint: Option<&Type>,
        span: Span,
    ) -> Result<Type, TypecheckerError> {
        // Types on both sides of the operation must be resolvable.
        let left = self.visit_expression(&mut binary_operation.left, type_hint)?;
        let right = self.visit_expression(&mut binary_operation.right, type_hint.or(Some(&left)))?;

        // Both of the types must be the same. If they are not, then we must error.
        if left != right {
            return Err(TypecheckerErrorKind::IncompatibleBinaryOperationTypes { left, right }.at(span));
        }

        // If the operator is comparing the two values, then the returned type is a boolean.
        if binary_operation.operator == BinaryOperator::Equals || binary_operation.operator == BinaryOperator::NotEquals
        {
            return Ok(Type::Boolean);
        }

        // TODO: Check if the operation is supported on the type. Some types do not support certain binary operations.
        //       To be safe, we whitelist integers.
        if matches!(left, Type::SignedInteger(_)) || matches!(left, Type::UnsignedInteger(_)) {
            return Ok(left);
        }

        Err(TypecheckerErrorKind::BinaryOperationNotSupported(left).at(span))
    }

    /// Checks and resolves the type of the provided [`FunctionCall`].
    fn visit_function_call(&mut self, function_call: &mut FunctionCall, span: Span) -> Result<Type, TypecheckerError> {
        let mut is_instance_call = false;

        let function_lookup_request = match &function_call.callee.kind {
            ExpressionKind::IdentifierReference(identifier) => {
                FunctionLookupRequest { name: identifier.clone(), owner_type_name: None, namespace: None }
            }

            ExpressionKind::NamespaceQualifier(namespace_qualifier) => FunctionLookupRequest {
                name: namespace_qualifier.identifier.clone(),
                owner_type_name: None,
                namespace: Some(namespace_qualifier.namespace.clone()),
            },

            ExpressionKind::MemberAccess(member_access) => {
                // The target of the member access expression must be a plain identifier.
                let member_access_target_name = match &member_access.target.kind {
                    ExpressionKind::IdentifierReference(name) => name,
                    _ => panic!(),
                };

                let (owner_type_name, namespace) =
                    if let Ok(variable_type) = self.typechecker.context.get_variable(member_access_target_name, span) {
                        is_instance_call = true;
                        match variable_type {
                            Type::Structure(id) => {
                                let structure = self.typechecker.context.get_declared_structure(id);
                                (structure.declared_name.clone(), structure.namespace.clone())
                            }

                            Type::Reference(inner) if matches!(inner.deref(), Type::Structure(_)) => {
                                let Type::Structure(id) = inner.deref() else { unreachable!() };
                                let structure = self.typechecker.context.get_declared_structure(id);
                                (structure.declared_name.clone(), structure.namespace.clone())
                            }

                            _ => return Err(TypecheckerErrorKind::UnsupportedFunctionCallee.at(span)),
                        }
                    } else if let Some(declared_type) =
                        self.typechecker.context.get_declared_type_by_name(member_access_target_name, span)
                    {
                        (declared_type.name.clone(), declared_type.namespace.clone())
                    } else {
                        return Err(TypecheckerErrorKind::UnsupportedFunctionCallee.at(span));
                    };

                FunctionLookupRequest {
                    owner_type_name: Some(owner_type_name),
                    name: member_access.name.clone(),
                    namespace: namespace,
                }
            }

            _ => return Err(TypecheckerErrorKind::UnsupportedFunctionCallee.at(span)),
        };

        // FIXME: I don't like the clone here.
        let checked_function =
            self.typechecker.context.get_checked_function(&function_lookup_request, span).cloned()?;

        function_call.resolved_callee = Some(checked_function.function_id);

        // If the function call is being done on an instance of the receiver type, then we must insert it as the first
        // parameter to the function (`this`).
        if is_instance_call {
            let target = match &function_call.callee.kind {
                ExpressionKind::MemberAccess(member_access) => member_access.target.clone(),
                _ => unreachable!(),
            };

            let reference_target = match self.visit_expression(target.clone().as_mut(), None)? {
                Type::Reference(_) => *target,
                _ => Expression::new(ExpressionKind::Reference(target), span),
            };

            function_call.arguments.insert(0, FunctionCallArgument { name: None, value: reference_target, span });
        }

        // The first check is easy, we just need to ensure that a sufficient number of arguments were passed in the
        // function call.
        if function_call.arguments.len() != checked_function.parameters.len() {
            return Err(TypecheckerErrorKind::FunctionCallArgumentSizeMismatch {
                name: function_lookup_request.name.clone(),
                expected: checked_function.parameters.len(),
                got: function_call.arguments.len(),
            }
            .at(span));
        }

        // By default, a function parameter is positional. A function parameter may also be named,
        // Positional function parameters must _always_ come before named function parameters.
        //
        // We will process arguments in two passes:
        // 1. The positional arguments. Each positional argument must match directly to the parameter
        //    of the same position.
        // 2. The named arguments. Each named argument must have a matching named parameter, and they
        //    can be defined in any order.
        //
        // FIXME: I don't know if this really belongs here, but it's the best place that I can put it for now. We
        //        are going to update the the function call to ensure that the Vec order of the arguments is
        //        the same order that the parameters are defined in.
        //
        //        This allows the codegen backend to just iterate over the arguments in order, without needing to
        //        check whether they match to the function's parameters.
        let mut ordered_arguments: Vec<FunctionCallArgument> = Vec::new();

        // Process the positional arguments. Each positional argument must have a matching parameter.
        for (idx, parameter) in checked_function.parameters.iter().filter(|it| !it.is_named).enumerate() {
            // A corresponding argument must exist, we checked the length of the `Vec`s above.
            let argument = &mut function_call.arguments[idx];
            if argument.name.is_some() {
                return Err(TypecheckerErrorKind::ExpectedPositionalFunctionCallArgument {
                    parameter_name: parameter.name.clone(),
                }
                .at(argument.span));
            }

            // For completeness sake, we can also provide the name in the function call argument beyond this point.
            argument.name = Some(parameter.name.clone());

            self.visit_function_call_argument(argument, parameter)?;
            ordered_arguments.push(argument.clone());
        }

        // Process the named arguments.
        for parameter in checked_function.parameters.iter().filter(|it| it.is_named) {
            let argument = function_call
                .arguments
                .iter_mut()
                .find(|it| it.name.as_ref().map(|it| it == &parameter.name).unwrap_or_default())
                .ok_or(
                    TypecheckerErrorKind::MissingFunctionCallArgument {
                        function_name: function_lookup_request.name.clone(),
                        parameter_name: parameter.name.clone(),
                    }
                    .at(span),
                )?;

            self.visit_function_call_argument(argument, parameter)?;
            ordered_arguments.push(argument.clone());
        }

        function_call.arguments = ordered_arguments;

        // All arguments have been type checked. The result of this function call is the return type of the function.
        Ok(checked_function.return_type)
    }

    /// Checks the type of a [`FunctionArgument`] against its matching [`FunctionParameter`].
    fn visit_function_call_argument(
        &mut self,
        argument: &mut FunctionCallArgument,
        parameter: &FunctionParameter,
    ) -> Result<(), TypecheckerError> {
        let argument_type = self.visit_expression(&mut argument.value, Some(&parameter.r#type))?;

        if argument_type != parameter.r#type {
            return Err(TypecheckerErrorKind::IncompatibleFunctionCallArgument {
                parameter_name: parameter.name.clone(),
                parameter_type: parameter.r#type.clone(),
                argument_type,
            }
            .at(argument.span));
        }

        Ok(())
    }

    /// Checks and resolves the type of the provided identifier reference.
    fn visit_identifier_reference(&mut self, name: &str, span: Span) -> Result<Type, TypecheckerError> {
        self.typechecker.context.get_variable(name, span).cloned()
    }

    /// Checks and resolves the type of the provided reference expression.
    fn visit_reference(&mut self, value: &mut Expression, _span: Span) -> Result<Type, TypecheckerError> {
        let inner = self.visit_expression(value, None)?;
        Ok(Type::Reference(inner.into()))
    }

    /// Checks and resolves the type of the provided dereference expression.
    fn visit_dereference(&mut self, value: &mut Expression, span: Span) -> Result<Type, TypecheckerError> {
        match self.visit_expression(value, None)? {
            Type::Reference(inner) => Ok(*inner),
            r#type => Err(TypecheckerErrorKind::InvalidDereference(r#type).at(span)),
        }
    }

    /// Checks and resolves any types in the provided structure initialization expression.
    fn visit_structure_initialization(
        &mut self,
        value: &mut StructureInitialization,
        type_hint: Option<&Type>,
        span: Span,
    ) -> Result<Type, TypecheckerError> {
        let structure_id = match type_hint {
            Some(Type::Structure(id)) => id,

            _ => {
                return Err(
                    TypecheckerErrorKind::StructureInitializationRequiresStructureType(type_hint.cloned()).at(span)
                );
            }
        };

        value.structure_id = Some(*structure_id);

        // FIXME: I don't like this `.clone()`.
        let structure = self.typechecker.context.get_declared_structure(structure_id).clone();

        // The structure initialization must have as many fields as the non-optional field count of the structure.
        let required_fields =
            structure.fields.iter().filter(|it| !matches!(it.r#type, Type::Optional(_))).collect::<Vec<_>>();
        if value.fields.len() < required_fields.len() || value.fields.len() > structure.fields.len() {
            return Err(TypecheckerErrorKind::StructureInitializationMissingFields {
                expected: required_fields.len(),
                got: value.fields.len(),
            }
            .at(span));
        }

        // We will then rewrite the structure initialization to have its fields ordered in declaration order.
        let mut ordered_fields: Vec<StructureInitializationField> = Vec::new();

        for (idx, declaration_field) in structure.fields.iter().enumerate() {
            // If there is not initialization field, then we can add our own field _if_ the type of the declaration is
            // an optional one.
            let mut initialization_field = match declaration_field.r#type {
                Type::Optional(_) => StructureInitializationField::new(
                    declaration_field.name.clone(),
                    Expression::new(OptionalEmpty::default().into(), span),
                    span,
                ),

                // TODO: Is this correct? I don't think it is. https://github.com/caoimhebyrne/petal/issues/2
                _ => value.fields[idx].clone(),
            };

            let value_type = self.visit_expression(&mut initialization_field.value, Some(&declaration_field.r#type))?;
            if value_type != declaration_field.r#type {
                return Err(TypecheckerErrorKind::IncompatibleTypes {
                    expected: declaration_field.r#type.clone(),
                    got: value_type,
                }
                .at(initialization_field.span));
            }

            ordered_fields.push(initialization_field);
        }

        value.fields = ordered_fields;

        Ok(Type::Structure(*structure_id))
    }

    /// Visits a member access expression.
    fn visit_member_access(&mut self, value: &mut MemberAccess, span: Span) -> Result<Type, TypecheckerError> {
        // The target of the access must be resolvable.
        let target_type = self.visit_expression(&mut value.target, None)?;

        debug!("Parsing member access expression for target type '{}' to member name '{}'", target_type, value.name);

        // We only support accessing members of structures at the moment.
        let structure_type = match target_type {
            Type::Structure(structure_id) => self.typechecker.context.get_declared_structure(&structure_id),
            _ => return Err(TypecheckerErrorKind::MemberAccessNotSupported.at(span)),
        };

        // The field must exist on the structure.
        let field = structure_type.fields.iter().find(|it| it.name == value.name).ok_or_else(|| {
            TypecheckerErrorKind::TypeDoesNotHaveMember { r#type: target_type, name: value.name.clone() }.at(span)
        })?;

        Ok(field.r#type.clone())
    }

    /// Visits an optional wrap expression.
    fn visit_optional_wrap(
        &mut self,
        optional_wrap: &mut OptionalWrap,
        type_hint: Option<&Type>,
        _span: Span,
    ) -> Result<Type, TypecheckerError> {
        // The hinted type might be an optional, and if it is, we should get the type it references, and use that as
        // the type hint for the expression that we are wrapping.
        let inner_type_hint = match type_hint {
            Some(Type::Optional(inner)) => Some(inner.as_ref()),

            _ => {
                trace!("Ignoring type hint '{type_hint:?}' for optional wrap expression");
                None
            }
        };

        let value_type = self.visit_expression(&mut optional_wrap.inner_value, inner_type_hint)?;
        optional_wrap.inner_type = value_type.clone();

        Ok(Type::Optional(value_type.into()))
    }

    /// Visits an optional empty expression.
    fn visit_optional_empty(
        &mut self,
        optional_wrap: &mut OptionalEmpty,
        type_hint: Option<&Type>,
        _span: Span,
    ) -> Result<Type, TypecheckerError> {
        // The hinted type might be an optional, and if it is, we should get the type it references, and use that as
        // the type hint for the expression that we are wrapping.
        let inner_type_hint = match type_hint {
            Some(Type::Optional(inner)) => inner.as_ref(),
            _ => return Ok(Type::Optional(Type::Unknown.into())),
        };

        optional_wrap.inner_type = inner_type_hint.clone();
        Ok(Type::Optional(inner_type_hint.clone().into()))
    }

    /// Visits an optional has value expression.
    fn visit_optional_has_value(
        &mut self,
        optional_has_value: &mut OptionalHasValue,
        _span: Span,
    ) -> Result<Type, TypecheckerError> {
        let inner_type = match self.visit_expression(&mut optional_has_value.optional_value, None)? {
            Type::Optional(inner) => inner,
            _ => panic!("Expected an optional type"),
        };

        optional_has_value.inner_type = *inner_type.clone();
        Ok(Type::Boolean)
    }

    /// Visits an optional force unwrap expression.
    fn visit_optional_force_unwrap(
        &mut self,
        optional_force_unwrap: &mut OptionalForceUnwrap,
        _span: Span,
    ) -> Result<Type, TypecheckerError> {
        let inner_type = match self.visit_expression(&mut optional_force_unwrap.optional_value, None)? {
            Type::Optional(inner) => inner,
            _ => panic!("Expected an optional type"),
        };

        optional_force_unwrap.inner_type = *inner_type.clone();
        Ok(*inner_type)
    }

    /// Visits a namespace declaration.
    fn visit_namespace_declaration(
        &mut self,
        namespace_declaration: &mut NamespaceDeclaration,
        _span: Span,
    ) -> Result<(), TypecheckerError> {
        for statement in &mut namespace_declaration.body {
            self.visit_statement(statement)?;
        }

        Ok(())
    }
}
