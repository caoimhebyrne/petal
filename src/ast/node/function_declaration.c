#include "function_declaration.h"
#include "ast/node.h"
#include "core/parameter.h"
#include "core/type.h"
#include "util/defer.h"
#include "util/format.h"
#include "util/string_builder.h"
#include "util/vector.h"
#include <stdlib.h>

FunctionDeclarationNode* function_declaration_node_create(
    Position position,
    char* name,
    Type* return_type,
    ParameterVector parameters,
    NodeVector body
) {
    FunctionDeclarationNode* node = malloc(sizeof(FunctionDeclarationNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_FUNCTION_DECLARATION;
    node->header.position = position;
    node->name = name;
    node->return_type = return_type;
    node->parameters = parameters;
    node->body = body;

    return node;
}

char* function_declaration_node_to_string(FunctionDeclarationNode* node) {
    auto type_string defer(free_str) = type_to_string(node->return_type);

    auto parameter_builder = string_builder_create();
    if (string_builder_is_invalid(parameter_builder)) {
        return nullptr;
    }

    string_builder_append(&parameter_builder, '[');

    for (size_t i = 0; i < node->parameters.length; i++) {
        auto parameter_string defer(free_str) = parameter_to_string(vector_get(&node->parameters, i));
        string_builder_append_str(&parameter_builder, parameter_string);

        if (i < node->parameters.length - 1) {
            string_builder_append_str(&parameter_builder, ", ");
        }
    }

    string_builder_append(&parameter_builder, ']');

    auto parameters defer(free_str) = string_builder_finish(&parameter_builder);
    return format_string(
        "FunctionDeclarationNode { name = '%s', type = %s, parameters = %s }",
        node->name,
        type_string,
        parameters
    );
}

void function_declaration_node_destroy(FunctionDeclarationNode* node) {
    free(node->name);
    type_destroy(node->return_type);
    vector_destroy(node->parameters, parameter_destroy);
    vector_destroy(node->body, node_destroy);
}
