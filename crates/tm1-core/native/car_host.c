#include "tm1/arena_ground.h"
#include "tm1/arena_guns.h"
#include <stddef.h>
#include <string.h>

_Static_assert(sizeof(PlayerCar) == 0x4a4, "JP player layout");
_Static_assert(sizeof(AICar) == 0x560, "JP AI layout");
_Static_assert(offsetof(PlayerCar, motion) == 0x380, "JP motion offset");
extern s16 gFieldsLastFrame, gUpdateRate;
extern u32 gFrameCount;

void uasoundStopTurboBoost(void)
{
}
void uasoundPlayTurboBoost(s32 a, s32 b)
{
    (void)a;
    (void)b;
}

static const s16* trig;
// The tables are adjacent slices of the verified JP executable image.
#define ROT_TABLE (trig + (0x80184d58 - 0x80183f14) / 2)
#define ATAN_TABLE (trig + (0x80188d58 - 0x80183f14) / 2)
static u32 random_state;
s32 arena_rand(void)
{
    random_state = random_state * 1103515245u + 12345u;
    return (random_state >> 16) & 32767;
}
s32 rsin(s32 angle)
{
    s32 phase = (u32)angle & 4095;
    s32 index = phase & 1023;
    if (phase & 1024)
        index = 1024 - index;
    return (phase & 2048) ? -trig[index] : trig[index];
}
s32 ratan2(s32 y, s32 x)
{
    s32 ay = abs(y), ax = abs(x);
    if (!ay && !ax)
        return 0;
    s32 lo = ay < ax ? ay : ax, hi = ay < ax ? ax : ay;
    s32 idx = (lo & 0x7fe00000) ? lo / (hi >> 10) : (lo << 10) / hi;
    s32 angle = ATAN_TABLE[idx];
    if (ay >= ax)
        angle = 1024 - angle;
    if (x < 0)
        angle = 2048 - angle;
    return y < 0 ? -angle : angle;
}
void do_smoke(VECTOR3* pos)
{
    arena_effect(pos, 8);
}

static s32 sine(s16 angle)
{
    s32 value = ROT_TABLE[2 * (abs((int)angle) & 4095)];
    return angle < 0 ? -value : value;
}
static s32 cosine(s16 angle)
{
    return ROT_TABLE[2 * (abs((int)angle) & 4095) + 1];
}
static s32 mul(s32 a, s32 b)
{
    return (a * b) >> 12;
}
void RotMatrixYXZ(const SVECTOR* a, MATRIX* m)
{
    s32 sx = sine(a->vx), cx = cosine(a->vx);
    s32 sy = sine(a->vy), cy = cosine(a->vy);
    s32 sz = sine(a->vz), cz = cosine(a->vz);
    s32 yx = mul(sy, sx), cx2 = mul(cy, sx);
    m->m[0][0] = mul(cy, cz) + mul(yx, sz);
    m->m[0][1] = -mul(cy, sz) + mul(yx, cz);
    m->m[0][2] = mul(sy, cx);
    m->m[1][0] = mul(sz, cx);
    m->m[1][1] = mul(cz, cx);
    m->m[1][2] = -sx;
    m->m[2][0] = -mul(sy, cz) + mul(cx2, sz);
    m->m[2][1] = mul(cx2, cz) + mul(sy, sz);
    m->m[2][2] = mul(cy, cx);
}

static struct {
    u32 all;
} gCtlPad;

u8 GetCtlPad(s16 button, s16 which)
{
    s32 down;
    u32 v;
    s16 idx;

    down = 0;
    if (which == 2) {
        if (GetCtlPad(button, 1) || GetCtlPad(button, 0)) {
            down = 1;
        }
    } else {
        v = gCtlPad.all;
        if (which == 1) {
            v >>= 16;
        }
        idx = button - 1;
        switch (idx) {
        case 1:
            down = (v >> 0xB) & 1;
            break;
        case 0:
            down = (v >> 8) & 1;
            break;
        case 2:
            down = (v >> 2) & 1;
            break;
        case 3:
            down = v & 1;
            break;
        case 4:
            down = (v >> 0xC) & 1;
            break;
        case 5:
            down = (v >> 0xE) & 1;
            break;
        case 6:
            down = (v >> 0xF) & 1;
            break;
        case 7:
            down = (v >> 0xD) & 1;
            break;
        case 8:
            down = (v >> 3) & 1;
            break;
        case 9:
            down = (v >> 1) & 1;
            break;
        case 10:
            down = (v >> 4) & 1;
            break;
        case 11:
            down = (v >> 6) & 1;
            break;
        case 12:
            down = (v >> 7) & 1;
            break;
        case 13:
            down = (v >> 5) & 1;
            break;
        }
    }
    return down;
}

static int current_level = -1;

s32 shellGetCurrentLevel(void)
{
    return current_level;
}

void tm1_bind_level(int level)
{
    if (level == current_level) return;
    current_level = level;
    numSlickSpots = 0;
    if (level == LEVEL2_WAREHOUSE) {
        InitLevel2TriggerPoints();
        InitLevel2Bridges();
        InitLevel2Curbs();
        InitLevel2PotHoles();
    } else if (level == LEVEL3_FREEWAY) {
        InitLevel3TriggerPoints();
        InitLevel3Bridges();
        InitLevel3Curbs();
        InitLevel3PotHoles();
    } else if (level == LEVEL4_CITY_PARK) {
        InitLevel4TriggerPoints();
        InitLevel4Bridges();
        InitLevel4Curbs();
        InitLevel4PotHoles();
        InitLevel4SlickSpots();
    } else if (level == LEVEL5_CYBURBIA) {
        InitLevel5TriggerPoints();
        InitLevel5Bridges();
        InitLevel5Curbs();
        InitLevel5PotHoles();
    } else if (level == LEVEL6_ROOFTOPS) {
        InitLevel6TriggerPoints();
        InitLevel6Bridges();
        numCurbs = 0;
        numPotHoles = numCheckPotHoles = 0;
    } else {
        InitLevel1TriggerPoints();
        InitLevel1Bridges();
        InitLevel1PotHoles();
        numCurbs = 0;
    }
}

static void init_vehicle_stats(CarStats* stats, s32 kind)
{
    switch (kind) {
    case 10:
        InitIceCreamTruck(stats);
        break;
    case 30:
        InitSemiTruck(stats);
        break;
    case UA_BOSS_CAR:
        InitBossCar(stats);
        break;
    case UA_MONSTER_TRUCK:
        InitMonsterTruck(stats);
        break;
    case UA_HUMVEE:
        InitHumvee(stats);
        break;
    case UA_CORVETTE:
        InitCorvette(stats);
        break;
    case UA_MAD_MAX:
        InitMadMax(stats);
        break;
    case UA_LOW_RIDER:
        InitLowRider(stats);
        break;
    case UA_POLICE_CAR:
        InitPoliceCar(stats);
        break;
    case 60:
        InitLamborghini(stats);
        break;
    case 80:
        InitHarley(stats);
        break;
    case 90:
        InitDuneBuggy(stats);
        break;
    default:
        InitTaxiCab(stats);
        break;
    }
}

void tm1_player_init(u8* state, u32* seed, s32 kind)
{
    PlayerCar car = { 0 };
    random_state = *seed;
    gFieldsLastFrame = 3;
    gUpdateRate = 20;
    gFrameCount = 0;
    SetBounce(&car, 2, 2, 0, 1);
    *seed = random_state;
    InitTireInfo(car.tires);
    InitTireGroup(car.tires, current_level == LEVEL6_ROOFTOPS ? 11 : 0);
    uaPadInit(&car, 1);
    car.stats.unk4C = 1;
    InitControlPad(car.skid);
    car.stats.unk38 = 1;
    car.weap.cur = WEAPON_SPECIAL;
    car.weap.ammo[WEAPON_SPECIAL] = 20;
    car.weap.maxAmmo = 28;
    car.weap.unk48 = arena_player_special_cost(kind);
    car.weap.unk46 = 2;
    car.weap.ammo[0] = car.weap.totalAmmo = 2;
    car.uaIndex = kind;
    car.stats.unk00 = 1;
    car.stats.unk3C = 1;
    car.stats.unk100 = 100;
    car.stats.unkDC = 25;
    car.stats.unkE8 = 3;
    car.stats.unkFC = 100;
    car.stats.updateRate = 60;
    init_vehicle_stats(&car.stats, kind);
    car.stats.damageThreshold = 75;
    car.stats.unk78 = abs(car.stats.unk80) + abs(car.stats.unk84);
    RecomputeCarDeltas(&car.stats);
    car.stats.unk40 = car.stats.unk44;
    car.motion.mat2.m[0][0] = car.motion.mat2.m[1][1] = car.motion.mat2.m[2][2] = 4096;
    car.motion.mat = car.motion.mat2;
    if (current_level == LEVEL2_WAREHOUSE) {
        car.motion.pos.vx = -0x28a0;
        car.motion.pos.vy = -0x11d0;
        car.motion.rot.vz = car.motion.rot2.vz = 0xc1;
    } else if (current_level == LEVEL6_ROOFTOPS) {
        car.motion.pos.vx = 0x3cd8;
        car.motion.pos.vy = 0x250;
        car.motion.pos.vz = 0x1e50;
        car.motion.rot.vz = car.motion.rot2.vz = -0x1c7;
    } else if (current_level == LEVEL5_CYBURBIA) {
        car.motion.pos.vx = 0x1c20;
        car.motion.pos.vy = 0x2ee0;
        car.motion.rot.vz = car.motion.rot2.vz = 0x400;
    } else if (current_level == LEVEL4_CITY_PARK) {
        car.motion.pos.vx = 0x12c0;
        car.motion.pos.vy = -0x12c0;
    } else if (current_level == LEVEL3_FREEWAY) {
        car.motion.pos.vx = 0x1c20;
        car.motion.pos.vy = -0x2390;
        car.motion.rot.vz = car.motion.rot2.vz = 0x400;
    } else {
        car.motion.pos.vx = car.motion.pos.vy = 0x460;
        car.motion.rot.vz = car.motion.rot2.vz = 0x200;
    }
    memcpy(state, &car, sizeof(car));
}

void tm1_drive_step(u8* state, u8* opponent, u16 buttons, s16 fields, u32 frame, const s16* table, u32* seed)
{
    trig = table;
    random_state = *seed;
    PlayerCar car;
    memcpy(&car, state, sizeof(car));
    gCtlPad.all = buttons;
    gFrameCount = frame;
    gFieldsLastFrame = fields;
    gUpdateRate = 60 / fields;
    CarUpdateControlPad(&car);
    RecomputeCarDeltas(&car.stats);
    if (car.stats.updateRate != gUpdateRate) {
        car.motion.vel.vy = car.motion.vel.vy * car.stats.updateRate / gUpdateRate;
        car.motion.rotDelta.vz = car.motion.rotDelta.vz * car.stats.updateRate / gUpdateRate;
        car.stats.updateRate = gUpdateRate;
    }
    car.collision.unk12 = 0;
    car.flags[9] = 0;
    s16 closest = GetClosestTriggerPt(&car, 1);
    memcpy(&car.flags[30], &closest, 2);
    if (car.flags[16] && (s16)(car.stats.unk18 -= fields) <= 0)
        car.flags[16] = 0;
    car.flags[25] = current_level != LEVEL6_ROOFTOPS && !(car.flags[10] || car.flags[23] || car.flags[0]);
    s32 speed = car.motion.vel.vy;
    UpdateBearing(&car, 1);
    CarUpdateDeltas(&car, 1);
    if (car.flags[13])
        UpdateCarOnSlickSpot(&car, 1);
    else
        CarCheckDynamics(&car);
    car.speedDelta = car.motion.vel.vy - speed;
    CheckSlickSpots(&car, 1);
    CheckHealthStands(&car, 1);
    CalcTireCoordinates(&car, 1);
    CheckCurbs(&car, 1);
    CheckBridges(&car, 1);
    CheckCatapults(&car, 1);
    CheckBombDamage(&car, 1);
    CheckSpikeDamage(&car, 1);
    CheckPotHoles(&car, 1);
    if (car.flags[27]) {
        AICar target;
        if (opponent)
            memcpy(&target, opponent, sizeof(target));
        tm1_check_player_smash(&car, opponent ? &target : NULL);
        if (opponent)
            memcpy(opponent, &target, sizeof(target));
    }
    UpdateBounce(&car, 1);
    UpdateTirePositions(&car, 1);
    CarRotUpdate(&car, 1);
    CarTransUpdate(&car, 1);
    *seed = random_state;
    trig = NULL;
    memcpy(state, &car, sizeof(car));
}

#include "tm1/arena_ai.h"

s32 arena_sqrt(s32 value)
{
    u32 magnitude = value < 0 ? ~(u32)value : (u32)value;
    if (magnitude == 0)
        return 0;
    int even = __builtin_clz(magnitude) & ~1;
    s32 normalized = even >= 24 ? (s32)((u32)value << (even - 24)) : value >> (24 - even);
    int offset = (0x80184728 - 0x80183f14) / 2 + normalized - 64;
    return ((u32)(s32)trig[offset] << ((31 - even) / 2)) >> 12;
}
static PlayerCar* arena_player;
static AICar* arena_opponent;
PlayerCar* GetPlayerInfo(s16 idx)
{
    (void)idx;
    return arena_player;
}
AICar* GetAICarInfo(s16 idx)
{
    return &arena_opponent[idx - arena_opponent->playerIdx];
}
void GetPlayerPosition(s16 idx, VECTOR3* out)
{
    *out = GetPlayerInfo(idx)->motion.pos;
}
void tm1_ai_restore_battle(AICar*);

// AICARINI searches the primary start points cyclically from its random
// choice, then searches the remaining slots in their original order.
static s16 reserve_ai_start(u16* used)
{
    int count = triggerPtStartPts.n;
    int first = arena_rand() % count;
    for (int i = 0; i < 15; i++) {
        int slot = i < count ? (first + i) % count : i;
        if (!(*used & (1u << slot))) {
            *used |= 1u << slot;
            return triggerPtStartPts.start[slot];
        }
    }
    return 0;
}

void tm1_ai_step(struct CollisionScene* scene, u8* state, u8* target, const s16* table,
    u32* seed, s32 kind, u16* spawn_points, s16 fields, u32 frame)
{
    PlayerCar player;
    AICar car;
    memcpy(&player, target, sizeof(player));
    memcpy(&car, state, sizeof(car));
    arena_player = &player;
    arena_opponent = &car;
    trig = table;
    random_state = *seed;
    gFrameCount = frame;
    gFieldsLastFrame = fields;
    gUpdateRate = 60 / fields;
    tm1_use_scene(scene);
    tm1_ai_restore_battle(&car);
    if (spawn_points) {
        car.uaIndex = kind;
        car.stats.unk38 = 1;
        arena_ai_weapon_profile(&car);
        car.weap.ammo[WEAPON_SPECIAL] = 20;
        car.weap.ammo[0] = car.weap.totalAmmo = 2;
        car.weap.maxAmmo = kind == UA_BOSS_CAR ? 60 : 28;
        car.weap.unk48 = arena_player_special_cost(kind);
        car.flags[25] = 1;
        car.weap.cur = WEAPON_SPECIAL;
        car.weap.unk46 = 2;
        car.stats.unk3C = 1;
        car.stats.unk4C = 1;
        car.stats.damageThreshold = 75;
        car.stats.unkDC = 25;
        car.stats.unkE8 = 3;
        init_vehicle_stats(&car.stats, kind);
        car.stats.unkFC = car.stats.unk100 = GetAICarStrength();
        car.stats.unk78 = abs(car.stats.unk80) + abs(car.stats.unk84);
        RecomputeCarDeltas(&car.stats);
        car.stats.updateRate = 60;
        AICarInitProfiles(&car);
        car.stats.unk40 = car.stats.unk44;
        car.unk18 = 0x140;
        car.unk1C = 0xa0;
        car.unk20 = 0x258;
        car.unk24 = 0x3e8;
        car.unk28 = 0x7d0;
        car.motion.mat2.m[0][0] = car.motion.mat2.m[1][1] = car.motion.mat2.m[2][2] = 4096;
        car.motion.mat = car.motion.mat2;
        InitTireInfo(car.tires);
        InitTireGroup(car.tires, current_level == LEVEL6_ROOFTOPS ? 8 : 0);
        car.unk01 = 2;
        car.stats.triggerPt = reserve_ai_start(spawn_points);
        InitTriggerPoint(&car);
        InitControlPad(car.skid);
        SetBounce(&car, 2, 2, 0, 0);
        AICarOutOfBattle(&car);
        car.driving = 1;
    } else {
        arena_schedule_battle(&car, 1, car.unk41 != 0, fields);
        AICarUpdate(&car);
    }
    *seed = random_state;
    memcpy(state, &car, sizeof(car));
    memcpy(target, &player, sizeof(player));
    tm1_use_scene(NULL);
    trig = NULL;
    arena_player = NULL;
    arena_opponent = NULL;
}

#include "tm1/arena_collision.h"
void VectorNormal(VECTOR* in, VECTOR* out)
{
    s32 x = (s16)in->vx, y = (s16)in->vy, z = (s16)in->vz;
    u32 squared = (u32)(x * x) + (u32)(y * y) + (u32)(z * z);
    u32 magnitude = (s32)squared < 0 ? ~squared : squared;
    int even = magnitude ? __builtin_clz(magnitude) & ~1 : 32;
    s32 index = even >= 24 ? (s32)(squared << (even - 24)) : (s32)squared >> (24 - even);
    s32 factor = trig[(0x801848bc - 0x80183f14) / 2 + index - 64];
    int shift = ((31 - even) >> 1) & 31;
    out->vx = x * factor >> shift;
    out->vy = y * factor >> shift;
    out->vz = z * factor >> shift;
}
static void contact_phase(PlayerCar* player, AICar* ai, u8 is_player, s16 fields)
{
    CarMotion* m = is_player ? &player->motion : &ai->motion;
    CarCollision* col = is_player ? &player->collision : &ai->collision;
    if (is_player == 2) {
        tm1_age_effects();
        arena_move_missiles();
        arena_move_drops();
        arena_move_flames();
        arena_move_tasers();
        arena_move_bullets();
        arena_move_fragments();
        arena_animate_flames();
    } else {
        if (is_player && player->stats.unk38) {
            CheckHitDetection(is_player ? (Car*)player : (Car*)ai, is_player);
        }
        if ((s16)col->unk16 >= 3) {
            m->lastRot = m->rot;
            m->lastPos = m->pos;
            m->unkA8 = m->rot2;
            m->lastMat = m->mat2;
        }
        if (is_player) {
            arena_update_weapons((Car*)player, 1);
            player->flags[2] = fields * 760 / 100 * 32 < m->vel.vy
                && CalcMaxRotBeforeDrift((Car*)player, 1) < abs(m->rotDelta.vz)
                && !player->flags[13];
        }
    }
    if (is_player != 2)
        tm1_contact_setup(
            is_player ? GetPlayerCs3D(0) : GetAICs3D(ai->playerIdx), m, is_player ? &player->stats : &ai->stats);
}

void tm1_contacts(struct CollisionScene* scene, u8* player_state, u8* ai_state, u8 is_player,
    const s16* tables, u32* seed, s16 fields, u32 frame)
{
    PlayerCar player;
    AICar ai;
    memcpy(&player, player_state, sizeof(player));
    memcpy(&ai, ai_state, sizeof(ai));
    arena_player = &player;
    arena_opponent = &ai;
    trig = tables;
    random_state = *seed;
    gFieldsLastFrame = fields;
    gUpdateRate = 60 / fields;
    gFrameCount = frame;
    tm1_use_scene(scene);
    tm1_contact_setup(GetPlayerCs3D(0), &player.motion, &player.stats);
    tm1_contact_setup(GetAICs3D(0), &ai.motion, &ai.stats);
    contact_phase(&player, &ai, is_player, fields);
    if (!is_player) {
        arena_regenerate_pickups();
        UpdateNumPotHoles();
    }
    memcpy(player_state, &player, sizeof(player));
    memcpy(ai_state, &ai, sizeof(ai));
    *seed = random_state;
    tm1_use_scene(NULL);
    arena_player = NULL;
    arena_opponent = NULL;
    trig = NULL;
}


// Run one complete PlayGame actor sequence under the caller's core lock.
// The original routines can change other cars during collision and weapon updates.
void tm1_bout_step(struct CollisionScene* scene, u8* player_state, u8** opponents,
    s16 count, s16* battle, u16 buttons, s16 fields, u32 frame, const s16* tables, u32* seed)
{
    PlayerCar player;
    AICar cars[8];
    memcpy(&player, player_state, sizeof(player));
    for (int i = 0; i < count; i++)
        memcpy(&cars[i], opponents[i], sizeof(cars[i]));
    arena_player = &player;
    arena_opponent = cars;
    trig = tables;
    random_state = *seed;
    gFrameCount = frame;
    gFieldsLastFrame = fields;
    gUpdateRate = 60 / fields;
    tm1_use_scene(scene);
    tm1_load_battle(battle);
    tm1_contact_setup(GetPlayerCs3D(0), &player.motion, &player.stats);
    for (int i = 0; i < count; i++)
        tm1_contact_setup(GetAICs3D(i), &cars[i].motion, &cars[i].stats);
    contact_phase(&player, cars, 2, fields);
    if (!tm1_recharge_player(&player)) {
        *seed = random_state;
        AICar* crushed = NULL;
        for (int i = 0; i < count && player.flags[27]; i++)
            if (cars[i].uaIndex == player.stats.unk30)
                crushed = &cars[i];
        tm1_drive_step((u8*)&player, (u8*)crushed, buttons, fields, frame, tables, seed);
        trig = tables;
        tm1_contact_setup(GetPlayerCs3D(0), &player.motion, &player.stats);
        contact_phase(&player, cars, 1, fields);
    }
    arena_schedule_battle(cars, count, GetNumAICarsInBattle(), fields);
    for (int i = 0; i < count; i++) {
        AICarUpdate(&cars[i]);
        contact_phase(&player, &cars[i], 0, fields);
    }
    UpdateNumAICarsInBattle();
    arena_regenerate_pickups();
    tm1_regenerate_stands();
    UpdateNumPotHoles();
    tm1_store_battle(battle);
    *seed = random_state;
    memcpy(player_state, &player, sizeof(player));
    for (int i = 0; i < count; i++)
        memcpy(opponents[i], &cars[i], sizeof(cars[i]));
    tm1_use_scene(NULL);
    arena_opponent = NULL;
    arena_player = NULL;
    trig = NULL;
}
