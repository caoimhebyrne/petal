#include "binary_operation.h"
#include "ast/node.h"
#include "util/defer.h"
#include "util/format.h"
#include <stdlib.h>

BinaryOperationNode* binary_operation_node_create(Position position, Node* left, Operator operator, Node * right) {
    BinaryOperationNode* node = malloc(sizeof(BinaryOperationNode));
    if (!node) {
        return nullptr;
    }

    node->header.kind = NODE_KIND_BINARY_OPERATION;
    node->header.position = position;
    node->left = left;
    node->operator= operator;
    node->right = right;

    return node;
}

char* binary_operation_node_to_string(BinaryOperationNode* node) {
    defer(free_str) auto left_string = node_to_string(node->left);
    if (!left_string) {
        return nullptr;
    }

    defer(free_str) auto right_string = node_to_string(node->right);
    if (!right_string) {
        return nullptr;
    }

    return format_string(
        "BinaryOperationNode { left = %s, operator = '%s', right = %s }",
        left_string,
        operator_to_string(node->operator),
        right_string
    );
}

const char* operator_to_string(Operator operator) {
    switch (operator) {
    case OPERATOR_ADD:
        return "+";

    case OPERATOR_SUBTRACT:
        return "-";

    case OPERATOR_MULTIPLY:
        return "*";

    case OPERATOR_DIVIDE:
        return "/";
    }
}

void binary_operation_node_destroy(BinaryOperationNode* node) {
    node_destroy(node->left);
    node_destroy(node->right);
}
