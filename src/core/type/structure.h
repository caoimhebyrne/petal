#pragma once

#include "core/position.h"
#include "core/type/type.h"
#include "util/vector.h"

// A member within a structure.
typedef struct {
    // The name of this member.
    char* name;

    // The type for this member.
    Type* type;
} StructureMember;

// A vector of structure members.
typedef Vector(StructureMember) StructureMemberVector;

// Creates a new StructureMember
// Parameters:
// - name: The name of the member.
// - type: The type of the member.
StructureMember structure_member_create(char* name, Type* type);

// Destroys a StructureMember.
void structure_member_destroy(StructureMember member);

// A structure type, defined by the user.
typedef struct {
    union {
        Type header;
    };

    // The structure's members.
    StructureMemberVector members;
} StructureType;

// Creates a new StructureType with an empty vector of members.
// Parameters:
// - position: The position that this type occurred at within the source file.
StructureType* structure_type_create(Position position);
