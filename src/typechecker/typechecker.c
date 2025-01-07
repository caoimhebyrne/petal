#include "typechecker.h"
#include "ast/node.h"
#include "ast/node/function_declaration.h"
#include "ast/node/number_literal.h"
#include "ast/node/variable_declaration.h"
#include "core/diagnostic.h"
#include "core/type.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/vector.h"
#include <stdio.h>
#include <string.h>

// Forward declarations:
bool typechecker_check_statement(Typechecker* typechecker, Node* node);
bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node);
bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node);

Type* typechecker_check_expression(Typechecker* typechecker, Node* node);
Type* typechecker_check_number_literal(Typechecker* typechecker, NumberLiteralNode* node);

// Resolves a type.
// If the type could not be resolved, nullptr is returned.
Type* typechecker_resolve_type(Typechecker* typechecker, Type** type_reference);

Typechecker typechecker_create(NodeVector* nodes, DiagnosticVector* diagnostics) {
    return (Typechecker){
        .nodes = nodes,
        .diagnostics = diagnostics,
    };
}

bool typechecker_check(Typechecker* typechecker) {
    for (size_t i = 0; i < typechecker->nodes->length; i++) {
        auto node = vector_get(typechecker->nodes, i);
        if (!typechecker_check_statement(typechecker, node)) {
            return false;
        }
    }

    return true;
}

bool typechecker_check_statement(Typechecker* typechecker, Node* node) {
    switch (node->kind) {
    case NODE_KIND_FUNCTION_DECLARATION:
        return typechecker_check_function_declaration(typechecker, (FunctionDeclarationNode*)node);

    case NODE_KIND_VARIABLE_DECLARATION:
        return typechecker_check_variable_declaration(typechecker, (VariableDeclarationNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->position, format_string("unable to type-check node: '%s'", node_string))
        );

        return false;
    }
}

bool typechecker_check_function_declaration(Typechecker* typechecker, FunctionDeclarationNode* node) {
    // The function's return type must be resolvable.
    auto return_type = typechecker_resolve_type(typechecker, &node->return_type);
    if (!return_type) {
        return false;
    }

    // If the return type is OK, we can type check the function's body.
    for (size_t i = 0; i < node->body.length; i++) {
        auto body_node = vector_get(&node->body, i);
        if (!typechecker_check_statement(typechecker, body_node)) {
            return false;
        }
    }

    return true;
}

// FIXME: Improve equality check here, do not cast to ValueType.
bool typechecker_check_variable_declaration(Typechecker* typechecker, VariableDeclarationNode* node) {
    // The variable's expected type must be resolvable.
    auto variable_type = (ValueType*)typechecker_resolve_type(typechecker, &node->type);
    if (!variable_type) {
        return false;
    }

    // The variable's initial value must be resolvable.
    auto value_type = (ValueType*)typechecker_check_expression(typechecker, node->value);
    if (!value_type) {
        return false;
    }

    // The type of the variable must be the same as the value.
    if (variable_type->value_kind != value_type->value_kind) {
        auto variable_type_string defer(free_str) = type_to_string((Type*)variable_type);
        auto value_type_string defer(free_str) = type_to_string((Type*)value_type);

        vector_append(
            typechecker->diagnostics,
            diagnostic_create(
                value_type->header.position,
                format_string("expected type '%s', but got '%s'", variable_type_string, value_type_string)
            )
        );

        return false;
    }

    // The types are matching.
    return true;
}

Type* typechecker_check_expression(Typechecker* typechecker, Node* node) {
    switch (node->kind) {
    case NODE_KIND_NUMBER_LITERAL:
        return typechecker_check_number_literal(typechecker, (NumberLiteralNode*)node);

    default:
        auto node_string defer(free_str) = node_to_string(node);
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(node->position, format_string("unable to type-check node: '%s'", node_string))
        );

        return nullptr;
    }
}

Type* typechecker_check_number_literal(Typechecker* typechecker, NumberLiteralNode* node) {
    (void)typechecker;

    // All integer literals are i32 and all float literals are f64 for now.
    if (node->is_float) {
        return (Type*)type_create_value(node->header.position, VALUE_TYPE_KIND_F64);
    } else {
        return (Type*)type_create_value(node->header.position, VALUE_TYPE_KIND_I32);
    }
}

Type* typechecker_resolve_type(Typechecker* typechecker, Type** type_reference) {
    auto type = *type_reference;

    // If the type is already resolved, we don't need to do anything.
    if (type->kind != TYPE_KIND_UNRESOLVED) {
        return type;
    }

    // This type is unresolved.
    auto unresolved_type = (UnresolvedType*)type;

    // In order to resolve it, we need to see if it is a valid "value" type.
    auto value_type_kind = value_type_kind_from_string(unresolved_type->name);
    if (value_type_kind == VALUE_TYPE_KIND_INVALID) {
        vector_append(
            typechecker->diagnostics,
            diagnostic_create(type->position, format_string("unknown type: '%s'", unresolved_type->name))
        );

        return nullptr;
    }

    return (Type*)type_create_value(type->position, value_type_kind);
}
