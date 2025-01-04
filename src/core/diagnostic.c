#include "core/diagnostic.h"

Diagnostic diagnostic_create(Position position, char* message) {
    return (Diagnostic){position, message};
}

void diagnostic_destroy(Diagnostic diagnostic) {
    free(diagnostic.message);
}
