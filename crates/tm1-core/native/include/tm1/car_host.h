#pragma once
#include <stdlib.h>
#include "tm1/car_layout.h"
#include "tm1/TIMER.H"

void uasoundStopTurboBoost(void);
void uasoundPlayTurboBoost(s32 a, s32 b);
void RotMatrixYXZ(const SVECTOR *angles, MATRIX *matrix);
void mathMulTransVec(MATRIX *, VECTOR3 *, VECTOR3 *);
void InitIceCreamTruck(CarStats *);
void InitSemiTruck(CarStats *);
void InitMonsterTruck(CarStats*);
void tm1_check_player_smash(PlayerCar*, AICar*);
void InitHumvee(CarStats*);
void InitCorvette(CarStats*);
void InitMadMax(CarStats*);
void InitLowRider(CarStats*);
void InitLamborghini(CarStats *);
void InitHarley(CarStats *);
void InitDuneBuggy(CarStats *);
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

s32 arena_player_special_cost(s32 kind);
