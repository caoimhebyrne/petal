#include "codegen.h"
#include "../string/format_string.h"
#include <string.h>

const char* header = "// This file has been auto-generated.\n"
                     "// Any changes you make here will not be saved.\n"
                     "#include <stdint.h>\n"
                     "\n"
                     "// Type definitions:\n"
                     "typedef int32_t i32;\n"
                     "typedef float f32;\n"
                     "\n";

Codegen codegen_create(NodeStream node_stream) { return (Codegen){.node_stream = node_stream}; }

char* codegen_generate(Codegen* codegen) {
    StringBuilder builder;
    string_builder_initialize(&builder, 2);

    string_builder_append_string(&builder, (char*)header);

    bool should_generate_main_method = true;

    for (size_t i = 0; i < codegen->node_stream.length; i++) {
        Node* node = codegen->node_stream.data[i];
        if (node->node_type != NODE_FUNCTION_DECLARATION) {
            continue;
        }

        FunctionDeclarationNode* declaration = (FunctionDeclarationNode*)node;
        if (strcmp(declaration->name, "main") == 0) {
            should_generate_main_method = false;
            break;
        }
    }

    for (size_t i = 0; i < codegen->node_stream.length; i++) {
        Node* node = codegen->node_stream.data[i];
        codegen_generate_node(&builder, node, true, false);
    }

    if (should_generate_main_method) {
        LOG_DEBUG("codegen", "main method does not exist. generating it!");

        NodeStream stream;
        node_stream_initialize(&stream, 1);

        NumberLiteralNode* return_value = number_literal_node_create(0);
        ReturnNode* return_node = return_node_create((Node*)return_value);
        node_stream_append(&stream, (Node*)return_node);

        FunctionDeclarationNode* node = function_declaration_node_create("main", "i32", stream);
        codegen_generate_function_declaration(&builder, node);
    }

    return string_builder_finish(&builder);
}

void codegen_generate_node(StringBuilder* builder, Node* node, bool is_top_level, bool as_value) {
    switch (node->node_type) {
    case NODE_FUNCTION_CALL:
        codegen_generate_function_call(builder, (FunctionCallNode*)node, as_value);
        break;

    case NODE_VARIABLE_DECLARATION:
        codegen_generate_variable_declaration(builder, (VariableDeclarationNode*)node, is_top_level);
        break;

    case NODE_FUNCTION_DECLARATION:
        codegen_generate_function_declaration(builder, (FunctionDeclarationNode*)node);
        break;

    case NODE_IDENTIFIER_REFERENCE:
        codegen_generate_identifier_reference(builder, (IdentifierReferenceNode*)node);
        break;

    case NODE_NUMBER_LITERAL:
        codegen_generate_number_literal(builder, (NumberLiteralNode*)node);
        break;

    case NODE_RETURN:
        codegen_generate_return(builder, (ReturnNode*)node);
        break;

    default:
        LOG_TODO("codegen", "implement generation of %s", node_to_string(node));
        break;
    }
}

void codegen_generate_return(StringBuilder* builder, ReturnNode* node) {
    string_builder_append_string(builder, "return");
    if (node->value) {
        string_builder_append(builder, ' ');
        codegen_generate_node(builder, node->value, false, true);
    }
    string_builder_append(builder, ';');
    string_builder_append(builder, '\n');
}

void codegen_generate_number_literal(StringBuilder* builder, NumberLiteralNode* node) {
    char* value = format_string("%f", node->value);
    string_builder_append_string(builder, value);
}

void codegen_generate_identifier_reference(StringBuilder* builder, IdentifierReferenceNode* node) {
    string_builder_append_string(builder, node->name);
}

void codegen_generate_function_declaration(StringBuilder* builder, FunctionDeclarationNode* node) {
    string_builder_append_string(builder, format_string("// %s\n", function_declaration_node_to_string(node)));

    string_builder_append_string(builder, format_string("%s %s() {\n", node->return_type_name, node->name));

    for (size_t i = 0; i < node->function_body.length; i++) {
        Node* child_node = node->function_body.data[i];

        string_builder_append_string(builder, "    ");
        codegen_generate_node(builder, child_node, false, false);
    }

    string_builder_append(builder, '}');
    string_builder_append(builder, '\n');
    string_builder_append(builder, '\n');
}

void codegen_generate_variable_declaration(StringBuilder* builder, VariableDeclarationNode* node, bool is_top_level) {
    if (is_top_level) {
        string_builder_append_string(builder, "const ");
    }

    string_builder_append_string(builder, format_string("%s %s = ", node->type_name, node->name));
    codegen_generate_node(builder, node->value, is_top_level, true);
    string_builder_append(builder, ';');
    string_builder_append(builder, '\n');
}

void codegen_generate_function_call(StringBuilder* builder, FunctionCallNode* node, bool as_value) {
    string_builder_append_string(builder, format_string("%s()", node->name));

    if (!as_value) {
        string_builder_append(builder, ';');
        string_builder_append(builder, '\n');
    }
}
