#include "codegen.h"
#include "codegen/result.h"
#include "core/diagnostic.h"
#include "core/position.h"
#include "util/format.h"
#include "util/vector.h"

Codegen codegen_create(NodeVector* nodes, DiagnosticVector* diagnostics) {
    return (Codegen){
        nodes,
        diagnostics,
    };
}

CodegenResult codegen_generate(Codegen* codegen) {
    vector_append(
        codegen->diagnostics,
        diagnostic_create((Position){.length = 1}, format_string("code generation is not implemented yet"))
    );

    return (CodegenResult){.status = CODEGEN_RESULT_FAILURE};
}
