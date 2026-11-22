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


void s_create_bullet(s32 owner, VECTOR3* dir, VECTOR3* pos, s32 kind, s32 damage, s32 life, s32 arg6);
Bullet *arena_bullets(void);
void arena_move_bullets(void);
void arena_update_guns(Car *,u8);
void arena_update_ai_guns(AICar *);

typedef struct {
    VECTOR3 pos;
    Cs *cs;
    VECTOR3 *target;
    s32 unk2C, index, life, unk38, unk3C, damage, unk44, unk48, unk50;
    s16 unk56, model;
    struct {
        SVECTOR ring[7][3];
        u8 ends[6][2];
        int head, ring_index, delay, color;
        u8 visible;
    } trail;
} Missile;
Missile *arena_missiles(void);
int arena_hit_target(const VECTOR3*, int, int);
s32 CAR_HD(VECTOR3 *,s32,s32);
void arena_special_missiles(int kind, int owner, CarStats* stats, CarMotion* motion, CarWeap* pad, u8* flags);
void arena_launch_missile(CarStats *,CarMotion *,CarWeap *,u8 *,u8);
void arena_update_weapons(Car *,u8);
typedef struct {VECTOR3 pos; s32 life,power,kind,idx,type;} DropWeap;
typedef struct {
    s32 life, owner, target, visible;
    VECTOR3 points[16];
} ArenaTaser;
DropWeap *arena_drops(void);
ArenaTaser *arena_tasers(void);
s32 FindFreeDropWeap(void);
void arena_move_drops(void);
int arena_launch_flame(Cs*, int);
void arena_move_flames(void);
void arena_animate_flames(void);
void arena_move_tasers(void);
void arena_schedule_battle(AICar *,s16,s16,s16);
void arena_move_missiles(void);
void tm1_age_effects(void);
void arena_effect(VECTOR3 *,s32);

void arena_regenerate_pickups(void);

void arena_move_fragments(void);

void arena_ai_weapon_profile(AICar* car);

void tm1_load_battle(const s16*);
void tm1_store_battle(s16*);
s16 GetNumAICars(void);

u8 tm1_recharge_player(PlayerCar*);
void tm1_regenerate_stands(void);
