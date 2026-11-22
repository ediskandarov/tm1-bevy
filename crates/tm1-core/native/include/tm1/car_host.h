#pragma once
#include <stdlib.h>
#include "tm1/car_layout.h"
#include "tm1/TIMER.H"

void uasoundStopTurboBoost(void);
void uasoundPlayTurboBoost(s32 a, s32 b);
void RotMatrixYXZ(const SVECTOR *angles, MATRIX *matrix);
void mathMulTransVec(MATRIX *, VECTOR3 *, VECTOR3 *);
void InitTaxiCab(CarStats *);
void RecomputeCarDeltas(CarStats *);
void UpdateBearing(Car *, u8);
void CarUpdateDeltas(Car *, u8);
void CarCheckDynamics(PlayerCar *);
void CarRotUpdate(Car *, u8);
void CarTransUpdate(Car *, u8);
u8 GetCtlPad(s16 button, s16 which);
void CarUpdateControlPad(PlayerCar *);
void uaPadInit(PlayerCar *, s32);
