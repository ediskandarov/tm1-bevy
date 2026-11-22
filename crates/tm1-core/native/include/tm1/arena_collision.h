#pragma once
#include "tm1/arena_ai.h"
#include "tm1/HD.H"
#include "tm1/ua_kinds.h"
#include <string.h>
#define setVector(v,a,b,c) do { (v)->vx=(a); (v)->vy=(b); (v)->vz=(c); } while(0)
#define copyVector(a,b) setVector(a,(b)->vx,(b)->vy,(b)->vz)
void VectorNormal(VECTOR *,VECTOR *);
void mathMulVec(MATRIX *,VECTOR3 *,VECTOR3 *);
void mathMulTransVecShort(MATRIX *,SVECTOR *,VECTOR3 *);
s32 hdpPntBox(HdBox *,VECTOR3 *,s32 *,s32);
s32 hdpPntVolume(DbVolume *,VECTOR3 *,HdCsHit *,s32);
void CheckHitDetection(Car*,u8);
void CalcHitDynamics(Car*,HdCsHit*,u8);
u8 CheckCSHit(Car*,u8,Cs*,s32,s32);
void DoCsHitCalculations(Car*,u8,Car*,u8,s32,s16);
void RicochetOffObject(Car*,u8,u8,s32,s16);
void RotateCarsAwayFromCollision(CarMotion*,CarMotion*);
void SetCollisionBounce(Car*,u8,s32);
void SetCollisCount(CarCollision*);
void CheckIfLostAICar(Car *);
void BoundSAngle(s16 *);
Cs *GetPlayerCs3D(s16);
Cs *GetAICs3D(s16);
u8 uaIsCarCS(Cs*,u8*,s16*);
s16 uaGetCarMatID(s16,u8);
void carTakeHit(s16,s32,VECTOR3*,u8);
void tm1_move_drop_box(void);
void bulDispatchDamage(s32,s32,s32,s32,VECTOR3*,s32);
void carGetPickup(Car*,u8,s32,s32);
void UAeffectBarricade(s32,s32,s32);
PotHole *GetPotHoleDat(s16);
void do_simple_spark(VECTOR3*);
void do_puff(VECTOR3*);
void InitMonsterSmash(Car*,u8,Car*,u8,s16);
#define DIFFICULTY_EASY 0
#define DIFFICULTY_HARD 2
s32 uaGetDifficulty(void);
static inline u8 uaHasPlayerBeatThisLevel(void) { return 0; }
s32 uaIsCarMatID(s16,s8*,u16*);
void SetFXSheet(s32,s32,s32,s32);
void UAeffectActHitLights(Cs*,s32,u8*);
void UAeffectSetFreezeLight(Cs*);
void CheckCatapults(Car*,u8);
void CheckBombDamage(Car*,u8);
void CheckSpikeDamage(Car*,u8);
void tm1_contact_setup(Cs*,const CarMotion*,const CarStats*);
struct CollisionScene;
void tm1_use_scene(struct CollisionScene *scene);
