#pragma once
#include "tm1/arena_ground.h"
#include "tm1/AI_CAR.H"
#include "tm1/AICARINI.H"
#include "tm1/LEVELS.H"
#define ABS(x) ((x) >= 0 ? (x) : -(x))
#define UA_POLICE_CAR 50
#define UA_BOSS_CAR 130
s32 arena_sqrt(s32 value);
s32 CalcMaxRotBeforeDrift(Car *,u8);
s32 SmoothValue(s32,s32,s32);
void InitNoCollision(CarCollision *);
void BringBackDriftingCar(Car *,u8);
void InitPoliceCar(CarStats *);
PlayerCar *GetPlayerInfo(s16);
AICar *GetAICarInfo(s16);
void GetPlayerPosition(s16,VECTOR3 *);
static inline s16 GetClosestPlayer(VECTOR3 *p,s16 ignored) { (void)p; (void)ignored; return 0; }
static inline s32 GetAICarStrength(void) { return 100; }
s32 shellGetCurrentLevel(void);
static inline u8 uaUsingLanes(void) { return shellGetCurrentLevel() >= LEVEL2_WAREHOUSE && shellGetCurrentLevel() <= LEVEL5_CYBURBIA; }

typedef struct { s32 vx, vy, vz, pad; } VECTOR;
#define UA_NONE (-1)
