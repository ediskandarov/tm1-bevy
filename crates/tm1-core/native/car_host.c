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

static _Thread_local s32 difficulty = 1;
void tm1_bind_difficulty(s32 value)
{
    difficulty = value;
}

s32 uaGetDifficulty(void)
{
    return difficulty;
}

s32 GetAICarStrength(void)
{
    static const s32 strength[] = { 60, 100, 120 };
    return strength[difficulty];
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

void InitHeliCar(CarStats* stats)
{
    stats->unk9C = 5;
    stats->unkC4 = 5;
    stats->unk88 = 0x12C;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x67;
    stats->unk98 = stats->unk88 / 3;
    stats->unkC8 = 4;
    stats->unkE0 = 0x19;
    stats->unkE4 = 5;
    stats->unkE8 = 4;
    stats->unk6C = 0xF;
    stats->unk70 = 0x25;
    stats->unk74 = 0x30;
    stats->unk7C = 0x52;
    stats->unk80 = 0x3F;
    stats->unk84 = -0x4B;
    stats->unk110 = -0x1D;
    stats->unk112 = 0x2B;
    stats->unk114 = 0xF;
    stats->dropPower = 0;
    stats->unk11A = 0;
    stats->unk11C = 0x2C;
    stats->unk10C = 0x320;
    stats->unk108 = 2;
    stats->unk04 = 2;
    stats->unk44 = 0x6E;
    stats->unk50 = 0x32;
    stats->unk104 = 0x1E;
    stats->unk1E = 0xA;
    stats->unk5C = 0x1A;
    stats->unk24 = 0x320;
}

static void init_vehicle_stats(CarStats* stats, s32 kind)
{
    switch (kind) {
    case UA_NONE:
        InitHeliCar(stats);
        break;
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

static void set_player_start(PlayerCar* car)
{
    setVector(&car->motion.pos, 0, 0, 0);
    setVector(&car->motion.rot, 0, 0, 0);
    if (current_level == LEVEL2_WAREHOUSE) {
        car->motion.pos.vx = -0x28a0;
        car->motion.pos.vy = -0x11d0;
        car->motion.rot.vz = car->motion.rot2.vz = 0xc1;
    } else if (current_level == LEVEL6_ROOFTOPS) {
        car->motion.pos.vx = 0x3cd8;
        car->motion.pos.vy = 0x250;
        car->motion.pos.vz = 0x1e50;
        car->motion.rot.vz = car->motion.rot2.vz = -0x1c7;
    } else if (current_level == LEVEL5_CYBURBIA) {
        car->motion.pos.vx = 0x1c20;
        car->motion.pos.vy = 0x2ee0;
        car->motion.rot.vz = car->motion.rot2.vz = 0x400;
    } else if (current_level == LEVEL4_CITY_PARK) {
        car->motion.pos.vx = 0x12c0;
        car->motion.pos.vy = -0x12c0;
    } else if (current_level == LEVEL3_FREEWAY) {
        car->motion.pos.vx = 0x1c20;
        car->motion.pos.vy = -0x2390;
        car->motion.rot.vz = car->motion.rot2.vz = 0x400;
    } else {
        car->motion.pos.vx = car->motion.pos.vy = 0x460;
        car->motion.rot.vz = car->motion.rot2.vz = 0x200;
    }
    car->motion.rot2 = car->motion.rot;
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
    set_player_start(&car);
    memcpy(state, &car, sizeof(car));
}

// CAR.c CarUpdate and CTLPAD.c ctlpadSpecial, single-player branch.
// Return 1 when the second pad switches the driving state of all opponents.
static int update_cheat_controls(PlayerCar* car, u32 pads)
{
    CarStats* st = &car->stats;
    int toggle_ai = 0;
    u32 second = pads >> 16;
    if (pads == 0x30c9) {
        st->cheatA = 1;
        st->cheatTimer = 0;
    } else if (pads == 0xc035) {
        st->cheatB = 1;
        st->cheatTimer = 0;
    } else if (second == 0x30c9) {
        st->cheatA = 1;
        st->cheatTimer = 0;
    }
    if (st->cheatA) {
        if (++st->cheatTimer >= 31) {
            st->cheatA = 0;
            st->cheatTimer = 0;
        }
        if (pads == 0xc036 || second == 0xc036) {
            if (st->cheatAArmed) {
                if (pads == 0xc036)
                    st->unk4C = !st->unk4C;
                else
                    toggle_ai = 1;
                st->cheatAArmed = 0;
                st->cheatTimer = 0;
            }
        } else if (pads == 0xc035 || pads == 0x3000) {
            if (st->cheatAArmed) {
                if (pads == 0x3000)
                    car->weap.reset = !car->weap.reset;
                else if (st->lostTimer)
                    st->lostTimer = 0;
                else {
                    st->lostTimer = 1;
                    car->flags[26] = 1;
                }
            }
            st->cheatAArmed = 0;
            st->cheatTimer = 0;
        } else {
            st->cheatAArmed = 1;
        }
    }
    if (st->cheatB) {
        if (++st->cheatTimer >= 31) {
            st->cheatB = 0;
            st->cheatTimer = 0;
        }
        if (pads == 0x30ca) {
            if (st->cheatBArmed) {
                int heavy = st->unk104 >= 101;
                st->unk104 = heavy ? 50 : 200;
                st->unk108 = heavy ? 2 : 4;
                st->cheatBArmed = 0;
                st->cheatTimer = 0;
            }
        } else {
            st->cheatBArmed = 1;
        }
    }
    return toggle_ai;
}

void tm1_drive_step(u8* state, u8* opponent, u32 buttons, s16 fields, u32 frame, const s16* table, u32* seed)
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
    if (car.flags[0])
        UpdateCarDeath(&car, 1);
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

static void init_camera_modes(PlayerCar* car, int fov)
{
    car->vrPos[1].vx = 0;
    car->vrPos[1].vy = -240;
    car->vrPos[1].vz = 112;
    car->vrRot[1].vx = 182;
    car->vrRot[1].vy = 0;
    car->vrRot[1].vz = 0;
    car->vrPos[5].vx = 0;
    car->vrRot[5].vy = 0;
    car->vrRot[5].vz = 0;
    car->vrRot[5].vx = 68;
    switch (car->uaIndex) {
    case UA_ICE_CREAM_TRUCK:
        car->vrPos[5].vy = -240;
        car->vrPos[5].vz = 112;
        break;
    case UA_SEMI_TRUCK:
        car->vrPos[5].vy = -256;
        car->vrPos[5].vz = 96;
        break;
    case UA_TAXI_CAB:
        car->vrPos[5].vy = -208;
        car->vrPos[5].vz = 72;
        break;
    case UA_MONSTER_TRUCK:
        car->vrPos[5].vy = -200;
        car->vrPos[5].vz = 64;
        break;
    case UA_LAMBORGHINI:
        car->vrPos[5].vy = -152;
        car->vrPos[5].vz = 48;
        break;
    case UA_POLICE_CAR:
        car->vrPos[5].vy = -192;
        car->vrPos[5].vz = 72;
        break;
    case UA_DUNE_BUGGY:
        car->vrPos[5].vy = -136;
        car->vrPos[5].vz = 48;
        break;
    case UA_HUMVEE:
        car->vrPos[5].vy = -152;
        car->vrPos[5].vz = 56;
        break;
    case UA_LOW_RIDER:
        car->vrPos[5].vy = -208;
        car->vrPos[5].vz = 48;
        break;
    case UA_MAD_MAX:
        car->vrPos[5].vy = -176;
        car->vrPos[5].vz = 48;
        break;
    case UA_HARLEY:
        car->vrPos[5].vy = -240;
        car->vrPos[5].vz = 48;
        break;
    case UA_CORVETTE:
        car->vrPos[5].vy = -168;
        car->vrPos[5].vz = 48;
        break;
    case UA_BOSS_CAR:
    default:
        break;
    }
    if (fov < 60) {
        car->vrPos[1].vy = 140 * car->vrPos[1].vy / 100;
        car->vrRot[1].vx = 100 * car->vrRot[1].vx / 140;
    }
    car->vrPos[2].vy = 155 * car->vrPos[1].vy / 100;
    car->vrPos[2].vz = 110 * car->vrPos[1].vz / 100;
    car->vrRot[2].vx = 100 * car->vrRot[1].vx / 140;
    car->vrPos[0].vx = (s16)car->stats.dropPower;
    car->vrPos[0].vy = car->stats.unk11A;
    car->vrPos[0].vz = car->stats.unk11C;
    car->vrRot[0].vx = 0;
    car->vrRot[0].vy = 0;
    car->vrRot[0].vz = 0;
    car->vrPos[3].vx = 0;
    car->vrPos[3].vy = -640;
    car->vrPos[3].vz = 320;
    car->vrRot[3].vx = 0;
    car->vrRot[3].vy = 0;
    car->vrRot[3].vz = 0;
}

// CAR_INIT.c CarInitVRModes and CAR.c CarViewUpdate (one-player views).
int tm1_update_car_view(const u8* state, int mode, int snap, s32* offset, s32* pitch)
{
    PlayerCar car;
    memcpy(&car, state, sizeof(car));
    init_camera_modes(&car, 60);
    const VECTOR3* target = &car.vrPos[mode - 1];
    int settled = abs(offset[1] - target->vy) < 17 || snap;
    int side = target->vx;
    if (mode != 1) {
        int limit = mode == 2 ? 112 : 160;
        int turn = car.motion.rotDelta.vz;
        int max_turn = car.stats.unkD4;
        side = turn > max_turn ? limit : turn < -max_turn ? -limit : turn * limit / max_turn;
    }
    offset[0] = snap || (mode == 1 && settled) ? side : SmoothValue(offset[0], side, 90);
    offset[1] = settled ? target->vy : SmoothValue(offset[1], target->vy, 90);
    offset[2] = settled ? target->vz : SmoothValue(offset[2], target->vz, 90);
    int target_pitch = car.vrRot[mode - 1].vx;
    *pitch = settled ? target_pitch : SmoothValue(*pitch, target_pitch, 90);
    return car.stats.unk3C && !(mode == 1 && settled);
}

static void init_motion_flags(u8* flags)
{
    flags[0] = 0;
    flags[1] = 0;
    flags[2] = 0;
    flags[3] = 0;
    flags[4] = 0;
    flags[6] = 0;
    flags[7] = 0;
    flags[8] = 0;
    flags[9] = 0;
    flags[10] = 0;
    flags[11] = 0;
    flags[12] = 0;
    flags[13] = 0;
    flags[14] = 0;
    flags[15] = 0;
    flags[16] = 0;
    flags[17] = 0;
    flags[18] = 0;
    flags[19] = 0;
    flags[20] = 0;
    flags[21] = 0;
    flags[22] = 0;
    flags[23] = 0;
    flags[24] = 0;
    flags[25] = 1;
    flags[26] = 0;
    flags[27] = 0;
    flags[29] = 0;
}

// CAR_INIT.c CarInit(which=0) on single-player stages.
static void respawn_player(PlayerCar* car, int fov)
{
    CarStats* stats = &car->stats;
    stats->unk00 = 1;
    init_motion_flags(car->flags);
    car->unk98 = car->unk99 = 0;
    setVector(&car->motion.vel, 0, 0, 0);
    set_player_start(car);
    setVector(&car->motion.rotDelta, 0, 0, 0);
    setVector(&car->motion.rot2Delta, 0, 0, 0);
    car->motion.rot2 = car->motion.rot;
    stats->unk38 = 1;
    InitNoCollision(&car->collision);
    MATRIX identity = { .m = { {4096, 0, 0}, {0, 4096, 0}, {0, 0, 4096} } };
    car->motion.mat = car->motion.mat2 = identity;
    stats->unkFC = stats->unk100 = 100;
    RecomputeCarDeltas(stats);
    stats->unkDC = 25;
    stats->unkE8 = 3;
    stats->unk34 = 0;
    stats->unk3C = 1;
    stats->damageThreshold = 75;
    stats->unk18 = 0;
    stats->spikeTimer = 0;
    stats->unk1C = 0;
    stats->blasts = 0;
    stats->unk22 = 0;
    stats->lostTimer = 0;
    stats->unk16 = 0;
    stats->unk03 = 0;
    stats->monster = 0;
    init_vehicle_stats(stats, car->uaIndex);
    stats->unk78 = abs(stats->unk80) + abs(stats->unk84);
    RecomputeCarDeltas(stats);
    stats->updateRate = 60;
    stats->unk40 = stats->unk44;
    InitTireInfo(car->tires);
    InitTireGroup(car->tires, current_level == LEVEL6_ROOFTOPS ? 11 : 0);
    s16 closest = GetClosestTriggerPt(car, 1);
    memcpy(&car->flags[30], &closest, sizeof(closest));
    car->weap.ammo[WEAPON_SPECIAL] = 20;
    car->weap.gunHeat = car->weap.gunOverheat = 0;
    init_camera_modes(car, fov);
    car->unkC0 = 0;
    SetBounce(car, 2, 2, 0, 1);
    car->speedDelta = 0;
    InitControlPad(car->skid);
    uaPadInit(car, 1);
    stats->cheatA = stats->cheatAArmed = 0;
    stats->cheatB = stats->cheatBArmed = 0;
    stats->unk01 = 0;
}

void tm1_campaign_player(u8* state, const s16* table, u32* seed, int new_life)
{
    PlayerCar car;
    memcpy(&car, state, sizeof(car));
    trig = table;
    random_state = *seed;
    gFieldsLastFrame = 3;
    gUpdateRate = 20;
    respawn_player(&car, 60);
    if (new_life) {
        car.stats.unkF8 = 0;
        car.weap.gunDelay = car.weap.fireDelay = 0;
        car.weap.unk04 = 0;
        memset(car.weap.ammo, 0, sizeof(car.weap.ammo));
        car.weap.cur = WEAPON_SPECIAL;
        car.weap.ammo[WEAPON_SPECIAL] = 20;
        car.weap.ammo[0] = car.weap.totalAmmo = 2;
        car.weap.maxAmmo = car.uaIndex == UA_BOSS_CAR ? 60 : 28;
        car.weap.unk48 = arena_player_special_cost(car.uaIndex);
        car.weap.unk46 = 2;
    }
    memcpy(state, &car, sizeof(car));
    *seed = random_state;
    trig = NULL;
}

// UA.c uaInitBossCar, after slot 3 receives its default weapons.
void tm1_place_boss(u8* state, u8* target, const s16* table, u32* seed, int fov)
{
    AICar boss;
    PlayerCar player;
    memcpy(&boss, state, sizeof(boss));
    memcpy(&player, target, sizeof(player));
    trig = table;
    random_state = *seed;
    int lower_group = player.unkC0 < 6;
    boss.stats.triggerPt = lower_group ? 28 : 4;
    InitTriggerPoint(&boss);
    boss.motion.pos.vz = lower_group ? 8000 : 7760;
    setVector(&player.motion.vel, 0, 0, 0);
    setVector(&player.motion.rotDelta, 0, 0, 0);
    if (player.motion.pos.vz < 6400)
        respawn_player(&player, fov);
    memcpy(state, &boss, sizeof(boss));
    memcpy(target, &player, sizeof(player));
    *seed = random_state;
    trig = NULL;
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
        if (kind != UA_NONE) arena_ai_weapon_profile(&car);
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
        car.unk18 = kind == UA_NONE ? 0x320 : 0x140;
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
        if (kind == UA_NONE) {
            car.stats.triggerPt = GetClosestTriggerPt(&car, 0);
            InitTriggerPoint(&car);
            car.unk01 = 0;
        }
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

// UA.c InitHelicoptorPosition / UpdateCamera and CAR.c CarViewUpdate.
void tm1_heli_camera(u8* state, const u8* target, const s16* table,
    s16 fields, u32 frame, int enter, s32* pose)
{
    PlayerCar player;
    AICar heli;
    memcpy(&player, target, sizeof(player));
    memcpy(&heli, state, sizeof(heli));
    arena_player = &player;
    trig = table;
    gFieldsLastFrame = fields;
    gUpdateRate = 60 / fields;
    gFrameCount = frame;
    VECTOR3 offset = { 0, -640, 320 }, out;
    if (enter) {
        mathMulTransVec(&player.motion.mat, &offset, &out);
        heli.unk01 = 0;
        heli.heliFlag = 1;
        setVector(&heli.motion.pos, player.motion.pos.vx + out.vx,
            player.motion.pos.vy + out.vy, player.motion.pos.vz + out.vz);
    }
    setVector(&offset, 0, 100, 0);
    mathMulTransVec(&player.motion.mat2, &offset, &out);
    s32 dx = player.motion.pos.vx + out.vx - heli.motion.pos.vx;
    s32 dy = player.motion.pos.vy + out.vy - heli.motion.pos.vy;
    s32 dz = player.motion.pos.vz + out.vz - heli.motion.pos.vz;
    pose[3] = -ratan2(dz, arena_sqrt(dx * dx + dy * dy));
    pose[4] = ratan2(dx, dy);
    if (!enter) {
        AICarHeliUpdate(&heli);
        heli.motion.pos.vz = SmoothValue(heli.motion.pos.vz, player.motion.pos.vz + 600, 95);
    }
    pose[0] = heli.motion.pos.vx;
    pose[1] = heli.motion.pos.vy;
    pose[2] = heli.motion.pos.vz;
    memcpy(state,&heli,sizeof(heli));
    arena_player = NULL;
    trig = NULL;
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
    s16 count, s16* battle, u32 buttons, u8 cheat_controls, s16 fields, u32 frame, const s16* tables, u32* seed)
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
        if (cheat_controls && update_cheat_controls(&player, buttons)) {
            u8 driving = !cars[0].driving;
            for (int i = 0; i < count; i++)
                cars[i].driving = driving;
        }
    }
    tm1_start_death(&player, 1);
    arena_schedule_battle(cars, count, GetNumAICarsInBattle(), fields);
    for (int i = 0; i < count; i++) {
        AICarUpdate(&cars[i]);
        contact_phase(&player, &cars[i], 0, fields);
        tm1_start_death(&cars[i], 0);
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
