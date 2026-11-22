#ifndef TM1_HOST_MATH_H
#define TM1_HOST_MATH_H
#include "common.h"

typedef struct {
    s16 m[3][3];
    s32 t[3];
} MATRIX;

typedef struct {
    s32 vx, vy, vz;
} VECTOR3;

typedef struct {
    s16 vx, vy, vz, pad;
} SVECTOR;

#define FXP_ONE 4096
s32 SquareRoot0(s32 value);
_Static_assert(sizeof(MATRIX) == 32, "MATRIX must preserve the PS1 layout");
_Static_assert(sizeof(VECTOR3) == 12, "VECTOR3 must preserve the PS1 layout");
_Static_assert(sizeof(SVECTOR) == 8, "SVECTOR must preserve the PS1 layout");
#endif
