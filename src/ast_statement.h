#pragma once

#include "array.h"
#include "ast_type.h"

/**
 * The AST of Petal is made up of statements and expressions.
 *
 * A statement is typically a node that does not produce a value, for example: a function declaration.
 */
typedef struct Statement Statement;

DEFINE_ARRAY_TYPE(StatementArray, statement_array, Statement*)

/**
 * Represents the different kinds of statements that are available.
 */
typedef enum {
    /**
     * A function declaration statement.
     */
    STATEMENT_KIND_FUNCTION_DECLARATION,
} StatementKind;

/**
 * A parameter as defined in a function declaration.
 */
typedef struct {
    /**
     * The name of the parameter.
     */
    StringBuffer name;

    /**
     * The value type of the parameter.
     */
    Type type;
} FunctionParameter;

DEFINE_ARRAY_TYPE(FunctionParameterArray, function_parameter_array, FunctionParameter)

/**
 * A function declaration statement.
 */
typedef struct {
    /**
     * The name of the function being declared.
     */
    StringBuffer name;

    /**
     * The parameters of the function.
     */
    FunctionParameterArray parameters;

    /**
     * The return type of this function.
     */
    Type return_type;

    /**
     * The body of the function.
     */
    StatementArray body;
} FunctionDeclarationStatement;

/**
 * The statement struct was forward declared for the `StatementArray` type.
 */
struct Statement {
    /**
     * The kind of statement that this is.
     */
    StatementKind kind;

    union {
        /**
         * STATEMENT_KIND_FUNCTION_DECLARATION.
         */
        FunctionDeclarationStatement function_declaration;
    };
};
