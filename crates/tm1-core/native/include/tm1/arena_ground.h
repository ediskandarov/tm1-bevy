#pragma once
#include "tm1/car_host.h"
#include "tm1/TRIGPTS.H"
#include "tm1/BRIDGES.H"
#include "tm1/CARUPD.H"
void InitLevel2TriggerPoints(void);
void InitLevel2Bridges(void);
void InitLevel2Curbs(void);
void InitLevel2PotHoles(void);
void InitLevel1PotHoles(void);
void SetFullCarDrift(CarMotion *);
void SetNoCarDrift(CarMotion *, u8 *);
void BoundAngle(s32 *);
s32 SmoothAngleValue(s32,s32,s32);
void SetBounce(Car *,s32,s32,s32,u8);
void CarInitBounceDeltas(Car *,u8);
void UpdateBounce(Car *,u8);
s32 arena_rand(void);
#define RANDINT(v) (arena_rand() % (v))
s32 rsin(s32);
s32 ratan2(s32,s32);
void do_smoke(VECTOR3 *);
void explodeCreateFragments(s32,VECTOR3 *);
