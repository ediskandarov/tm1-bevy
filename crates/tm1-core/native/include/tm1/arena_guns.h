#pragma once
#include "tm1/arena_collision.h"
typedef struct Bullet {
    /* 0x00 */ VECTOR3 pos;
    /* 0x0C */ VECTOR3 prev;
    /* 0x18 */ VECTOR3 vel;
    /* 0x24 */ s32 owner;
    /* 0x28 */ s32 life;
    /* 0x2C */ s32 damage;
    /* 0x30 */ s32 unk30;
    /* 0x34 */ u8 kind;
    /* 0x35 */ u8 pad35[3];
    /* 0x38 */ s32 unk38;
} Bullet; /* 0x3C */


Bullet *arena_bullets(void);
void arena_move_bullets(void);
void arena_update_guns(Car *,u8);
void arena_update_ai_guns(AICar *);

typedef struct {
    VECTOR3 pos;
    Cs *cs;
    VECTOR3 *target;
    s32 unk2C, index, life, unk38, unk3C, damage, unk44, unk48, unk50;
    s16 unk56;
} Missile;
Missile *arena_missiles(void);
s32 CAR_HD(VECTOR3 *,s32,s32);
void arena_launch_missile(CarStats *,CarMotion *,CarWeap *,u8 *,u8);
void arena_update_weapons(Car *,u8);
typedef struct {VECTOR3 pos; s32 life,power,kind,idx,type;} DropWeap;
typedef struct {s32 life,target;} ArenaTaser;
DropWeap *arena_drops(void);
ArenaTaser *arena_tasers(void);
void arena_move_drops(void);
void arena_move_tasers(void);
void arena_summon_police(AICar *,s16);
void arena_move_missiles(void);
void tm1_age_effects(void);
void arena_effect(VECTOR3 *,s32);

void arena_regenerate_pickups(void);

void arena_move_fragments(void);
