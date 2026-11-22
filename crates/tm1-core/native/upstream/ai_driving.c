#include "tm1/arena_ai.h"
#include "tm1/arena_guns.h"

s16 numAICarsInBattle = 0;
static s16 aiCarsInBattle[2];

void AICarUpdate(AICar* car)
{
    s32 diff;
    s32 rot;
    s32 maxRot;
    s32 fl;
    s32 sp;

    if (car->stats.unk03 != 0) {
        car->stats.triggerPt = GetClosestTriggerPt(car, 0);
        InitTriggerPoint(car);
        car->stats.unk03 = 0;
    }
    RecomputeCarDeltas(&car->stats);
    if (car->stats.unk18 >= 0 && (car->stats.unk18 -= GetFieldsLastFrame()) <= 0) {
        car->flags[16] = 0;
        tm1_clear_freeze_light(0, car->playerIdx);
    }
    car->flags[25] = shellGetCurrentLevel() != LEVEL6_ROOFTOPS && !(car->flags[10] || car->flags[23] || car->flags[27] || car->flags[0]);
    car->collision.unk12 = 0;
    car->flags[0x09] = 0;
    AICarUpdateHealthTier(car);
    if (car->driving != 0 && car->flags[0x00] == 0) {
        UpdateBearing(car, 0);
        AICarUpdateClosestPlayer(car);
        AICarUpdatePlayerRange(car);
        AICarUpdateCurrPtRange(car);
        AICarUpdatePlayerDir(car);
        AICarUpdateDrivingProfile(car);
        arena_update_ai_guns(car);

    } else if (car->flags[0x00] != 0) {
        car->unk34 = 0;
    }
    AICarUpdateTransition(car);
    tm1_update_car_speed(car->uaIndex, car->motion.vel.vy);

    if (car->driving != 0) {
        CarRotUpdate(car, 0);
        CarTransUpdate(car, 0);
    }
    if (arena_bullets() && car->stats.unk38 && (car->unk40 || shellGetCurrentLevel() == LEVEL6_ROOFTOPS)) {
        CheckHitDetection(car, 0);
        if (car->collision.unk0 && abs(car->motion.rot.vz - car->motion.rot2.vz) >= 683)
            car->unk4B = 1;
    }
    fl = GetFieldsLastFrame();
    sp = car->motion.vel.vy;
    if ((((fl * 0x2F8) / 100) << 5) < sp && car->flags[0x0D] == 0) {
        rot = car->unk1C4;
        rot = ABS(rot);
        if (rot < 683 || car->unk160 == 0) {
            maxRot = abs(car->motion.rotDelta.vz);
            if (CalcMaxRotBeforeDrift(car, 0) < maxRot) {
                goto block_46;
            }
            goto block_47;
        }
    block_46:
        car->flags[0x02] = 1;
        return;
    }
block_47:
    car->flags[0x02] = 0;
}

void AICarUpdateHealthTier(AICar* car)
{
    if (car->stats.unk40 < car->unk104) {
        car->unk02 = 0;
    } else if (car->stats.unk40 < car->unk105) {
        car->unk02 = 1;
    } else if (car->stats.unk40 < car->unk106) {
        car->unk02 = 2;
    } else {
        car->unk02 = 3;
    }
}

void AICarUpdateClosestPlayer(AICar* car)
{
    car->unk0E = GetClosestPlayer(&car->motion.pos, -1);
}

void AICarUpdatePlayerRange(AICar* car)
{
    VECTOR3 pos;
    u16 dead0;
    u16 dead1;
    PlayerCar* info;

    GetPlayerPosition(car->unk0E, &pos);
    pos.vx -= car->motion.pos.vx;
    pos.vy -= car->motion.pos.vy;
    car->unk34 = arena_sqrt((pos.vx * pos.vx) + (pos.vy * pos.vy));
    info = GetPlayerInfo(car->unk0E);
    dead0 = car->unk172;
    dead1 = info->unkC0;

    if (car->unk4C == 0) {
        if (car->unk34 > car->unk38) {
            car->unk178 += GetFieldsLastFrame();
            car->unk17A = 0;
        } else if (car->unk34 < car->unk38) {
            car->unk178 = 0;
            car->unk17A += GetFieldsLastFrame();
        } else {
            car->unk178 = 0;
            car->unk17A = 0;
        }
    }
    if (car->flags[0x0B] == 0) {
        car->unk38 = car->unk34;
    }
    if (car->unk34 >= 6401) {
        car->unk40 = 0;
        if (car->heliFlag == 0) {
            AICarOutOfBattle(car);
        }
    } else {
        car->unk40 = 1;
        if (car->unk41 == 0) {
            AICarInBattle(car);
        }
        if (car->unk34 < car->unk18) {
            car->unk42 = 1;
        } else {
            car->unk42 = 0;
        }
        if (car->unk34 < car->unk1C) {
            car->unk43 = 1;
            car->unk44 = 1;
            car->unk45 = 1;
            car->unk46 = 1;
        } else {
            car->unk43 = 0;
            if (car->unk34 < car->unk20) {
                car->unk44 = 1;
                car->unk45 = 1;
                car->unk46 = 1;
            } else {
                car->unk44 = 0;
                if (car->unk34 < car->unk24) {
                    car->unk45 = 1;
                    car->unk46 = 1;
                } else {
                    car->unk45 = 0;
                    if (car->unk34 < car->unk28) {
                        if (car->unk46 == 0) {
                            car->flags[0x18] = 1;
                        }
                        car->unk46 = 1;
                    } else {
                        car->unk46 = 0;
                    }
                }
            }
        }
    }
}

void AICarUpdateCurrPtRange(AICar* car)
{
    VECTOR3 v;
    VECTOR3 out;
    VECTOR delta;
    u16 distance;

    distance = car->unk186;
    v.vy = 0;
    v.vz = 0;
    v.vx = (s16)distance;
    mathMulTransVec(&car->motion.mat2, &v, &out);
    delta.vx = (car->unk1A0 + out.vx) - car->motion.pos.vx;
    delta.vy = (car->unk1A4 + out.vy) - car->motion.pos.vy;
    car->unk1E4 = arena_sqrt((delta.vx * delta.vx) + (delta.vy * delta.vy));
    if (delta.vy == 0) {
        delta.vy = 1;
    }
    car->unk1EC = ratan2(delta.vx, delta.vy);
    BoundAngle(&car->unk1EC);
}

void AICarUpdateNextPtAngle(AICar* car)
{
    VECTOR delta;

    delta.vx = car->unk1A8 - car->motion.pos.vx;
    delta.vy = car->unk1AC - car->motion.pos.vy;
    car->unk1E8 = arena_sqrt((delta.vx * delta.vx) + (delta.vy * delta.vy));
    if (delta.vy == 0) {
        delta.vy = 1;
    }
    car->unk1C8 = ratan2(delta.vx, delta.vy);
    BoundAngle(&car->unk1C8);
}

void AICarUpdateDrivingProfile(AICar* car)
{
    s16 r;

    if (car->routeVal > 0) {
        car->routeVal = car->routeVal - GetFieldsLastFrame();
        return;
    }
    if (car->unk41 != 0 || numAICarsInBattle < 2) {
        if (car->chosen != 0) {
            if (car->unk01 == 0 && car->unk46 != 0) {
                car->chosen = 0;
            }
            car->unk01 = 0;
            car->routeVal = car->profile.route[car->unk02].val;
        } else {
            r = arena_rand() % 100;
            if (r < car->profile.route[car->unk02].prio) {
                car->unk01 = 0;
                car->routeVal = car->profile.route[car->unk02].val;
            } else if (r < car->profile.route[car->unk02].unk04) {
                car->unk01 = 1;
                car->routeVal = car->profile.route[car->unk02].unk10;
            } else {
                goto tier3;
            }
        }
    } else {
    tier3:
        car->unk01 = 2;
        car->routeVal = car->profile.route[car->unk02].unk14;
    }
}

void AICarInitTransition(AICar* car)
{
    s32 d;

    car->flags[0x1D] = 0;
    if (car->unk180 != 0) {
        car->stats.unk22++;
        if ((s16)car->stats.unk22 >= 7 && car->unk34 >= 3201) {
            car->flags[0x1D] = 1;
        }
    } else {
        car->stats.unk22 = 0;
    }
    if (triggerPtGroups.count >= 2 && car->flags[0x0A] == 0 && car->unk34 >= 3201) {
        d = abs(arena_group_height(car->unk176) - car->motion.pos.vz);
        if ((car->stats.unk6C * 2) < d) {
            car->stats.unk03 = 1;
            if (shellGetCurrentLevel() == LEVEL6_ROOFTOPS) {
                if (car->motion.pos.vx >= 0x15E0) {
                    if (car->motion.pos.vx < 0x2581) {
                        if (car->motion.pos.vy >= 0x1770) {
                            if (car->motion.pos.vy < 0x1839) {
                                car->stats.unk03 = 0;
                            }
                        }
                    }
                }
            }
        }
    }
    car->unk180 = 0;
    if (car->unk49 != 0) {
        car->unk49 = 0;
        car->unk10 = 0;
        car->stats.unkFC = car->stats.unk100;
        RecomputeCarDeltas(&car->stats);
    }
    car->unk178 = 0;
    car->unk17A = 0;
    if (car->unk17E >= 601 && car->unk34 >= 3201) {
        car->stats.unk03 = 1;
    }
    car->unk17E = 0;
}

void AICarUpdateTransition(AICar* car)
{
    InitControlPad(car->skid);
    if (car->heliFlag == 0 && car->flags[0x00] == 0) {
        car->unk17E = car->unk17E + GetFieldsLastFrame();
    }
    if (car->unk17E >= 601) {
        StartLostCheck(car);
    }
    if (car->unk49 != 0) {
        HelpLostCar(car);
    }
    if (car->unk16D != 0) {
        BringBackAICarInCollision(car);
    } else if (car->driving != 0 && car->flags[0x00] == 0) {
        if (car->flags[0x0D] != 0) {
            UpdateCarOnSlickSpot(car, 0);
        }
        if (car->unk160 != 0) {
            AICarTurn(car);
        } else {
            AICarDriveBetweenPts(car);
            AICarUpdateSwerve(car);
            if (car->unk41 != 0 && car->unk01 == 0 && car->unk178 > (s16)car->stats.unk1E
                && car->direction == DIRECTION_REAR && car->unk18A >= 81) {
                AIInitTurnFlags(car);
                GetNextTriggerPoint(car, car->unk170);
                AICarInitTransition(car);
                CalcTurnStart(car, 0);
            }
        }
        AICarUpdateControlPad(car);
        CarUpdateDeltas(car, 0);
    }
    CheckSlickSpots(car, 0);
    CheckHealthStands(car, 0);

    CalcTireCoordinates(car, 0);
    CheckCurbs(car, 0);
    CheckBridges(car, 0);

    CheckCatapults(car, 0);
    CheckBombDamage(car, 0);
    CheckSpikeDamage(car, 0);
    CheckPotHoles(car, 0);
    if (car->flags[27])
        tm1_check_ai_smash(car);

    UpdateBounce(car, 0);
    UpdateTirePositions(car, 0);
    if (car->flags[0])
        UpdateCarDeath(car, 0);
}

void NoBeadOnPlayer(AICar* car)
{
    if (car->heliFlag != 0 && car->uaIndex != UA_NONE) {
        GetNewPointToDriveTo(car);
        car->heliFlag = 0;
    }
}

void GetNewPointToDriveTo(AICar* car)
{
    s16 pt;
    u16 current;
    s32 point;

    pt = GetClosestTriggerPt(car, 0);
    current = car->unk172;
    do {
    } while (0);
    point = pt;
    if (point == (s16)current) {
        pt = -1;
    }
    GetNextTriggerPoint(car, pt);
    AICarInitTransition(car);
    CalcTurnStart(car, 0);
    do {
        car->unk160 = 1;
    } while (0);
    if (car->uaIndex != UA_NONE) {
        car->heliFlag = 0;
    }
    car->unk4B = 0;
    SetNoCarDrift(&car->motion, car->flags);
}

void HelpLostCar(AICar* car)
{
    car->unk10 += GetFieldsLastFrame();
    if (car->unk10 >= 181 || car->unk180 >= 4 || car->unk4D != 0) {
        GetNewPointToDriveTo(car);
        car->unk4D = 0;
    }
}

void AIInitTurnFlags(AICar* car)
{
    s32 angle;

    do {
        car->unk160 = 1;
        angle = 0x800;
    } while (0);
    do {
        car->unk161 = 0;
        car->unk1CC = angle;
    } while (0);
    do {
        car->unk1D0 = 0x7FFF;
        return;
    } while (0);
}

void AICarTurn(AICar* car)
{
    s32 diff;
    s32 t;
    s32 a;
    u8 turn;

    AICarUpdateNextPtAngle(car);
    turn = 0;
    if (GetBeadOnPlayer(car, car->unk1E8)) {
        diff = car->motion.rot.vz - car->unk30;
        car->heliFlag = 1;
        if ((s8)car->stats.unk04 >= GetPlayerInfo(car->unk0E)->stats.unk108
            && (s16)(arena_rand() % 100) < car->profile.attack[car->unk02].unk00) {
            car->unk48 = 1;
        } else {
            car->unk48 = 0;
        }
    } else {
        diff = car->motion.rot.vz - car->unk1C8;
        NoBeadOnPlayer(car);
    }
    BoundAngle(&diff);
    t = ABS(diff);
    if (t < 57) {
        turn = 1;
    } else if (t < 342) {
        a = car->unk1CC;
        if (a < 0) {
            a = -a;
        }
        if (a < t) {
            turn = 1;
        } else {
            turn = 0;
            car->unk1CC = diff;
        }
    }
    if (turn != 0) {
        UpdateCurrentTriggerPt(car);
        car->motion.unk00 = 0;
        if (car->unk16C != 0) {
            car->motion.vel.vy = car->stats.unkA4;
        }
        goto noDrift;
    }
    if (!TimeToComeOutOfTurn(car, diff)) {
        if (diff > 0) {
            car->skid[2] = 1;
        } else if (diff < 0) {
            car->skid[3] = 1;
        }
    } else {
        car->skid[3] = 0;
        car->skid[2] = 0;
    }
    if (car->unk16C != 0) {
        SetFullCarDrift(&car->motion);
        if (car->skid[2] != 0) {
            car->skid[7] = 1;
        } else if (car->skid[3] != 0) {
            car->skid[6] = 1;
        }
        goto tail;
    }
noDrift:
    if (car->flags[0x01] == 0) {
        SetNoCarDrift(&car->motion, car->flags);
    }
tail:
    if (car->motion.vel.vy > car->stats.unkB4) {
        car->skid[5] = 1;
        car->skid[4] = 0;
    } else {
        car->skid[4] = 1;
        car->skid[5] = 0;
    }
}

u8 TimeToComeOutOfTurn(AICar* car, s32 diff)
{
    s32 rot;
    s32 d;
    s32 dd;
    s32 hi;
    s32 lo;
    s32 spd;
    u8 res;

    rot = car->motion.rotDelta.vz;
    d = ABS(diff);
    if (rot < 0) {
        rot = -rot;
    }
    if (d >= (rot * 2)) {
        if (d >= 1025) {
            res = 0;
        } else if (d >= 57) {
            if (car->skid[6] != 0) {
                hi = car->stats.unkD4 * car->stats.unkE8;
            } else if (car->skid[7] != 0) {
                hi = car->stats.unkD4 * car->stats.unkE8;
            } else {
                hi = car->stats.unkD4 * 2;
            }
            dd = ABS(diff);
            lo = car->stats.unkD0 + (((hi - car->stats.unkD0) * dd) / 1024);
            if (car->motion.vel.vy < car->stats.unkA8) {
                spd = car->stats.unkA8;
            } else {
                spd = car->motion.vel.vy;
            }
            lo += ((hi - lo) * (spd - car->motion.vel.vy)) / spd;
            res = lo < abs((s16)car->motion.unk00);
        } else {
            res = 1;
        }
    } else {
        res = 1;
    }
    if (abs((s16)car->motion.unk00) < 5) {
        res = 0;
    }
    return res;
}

void AICarDriveBetweenPts(AICar* car)
{
    s32 dist;
    s32 spd;
    s32 t;
    s32 fields;
    s32 half;

    if (car->flags[0x01] == 0) {
        SetNoCarDrift(&car->motion, car->flags);
    }
    if (car->unk200 == 0) {
        UpdateTurnStart(car);
    }
    if (car->unk1E4 < car->unk1B4) {
        if (car->unk200 != 0) {
            GetNextTriggerPoint(car, -1);
            AICarInitTransition(car);
            CalcTurnStart(car, 1);
            car->unk200 = 0;
            return;
        }
        AIInitTurnFlags(car);
        return;
    }
    if (car->unk1E4 < car->unk200) {
        GetNextTriggerPoint(car, -1);
        AICarInitTransition(car);
        CalcTurnStart(car, 1);
        car->unk200 = 0;
    }
    if (car->heliFlag != 0) {
        if (car->unk48 != 0) {
            dist = car->unk34;
            spd = GetBufferZoneSpeed(car);
        } else {
            spd = 0;
            dist = car->unk34 - car->unk18;
        }
        car->unk161 = 0;
    } else {
        dist = car->unk1E4;
        spd = car->stats.unkB4;
    }
    if (dist <= 0) {
        s32 a = car->unk2C;
        a = ABS(a);
        if (a < 113) {
            goto set53;
        }
        if (car->uaIndex == UA_NONE) {
            goto set53;
        }
        car->skid[4] = 1;
        car->heliFlag = 0;
        return;
    }
    t = car->motion.vel.vy;
    t = ABS(t);
    do {
    } while (0);
    fields = GetFieldsLastFrame();
    if (t < ((fields * 380) / 100) || dist > 0x1FFFFF) {
        goto set52;
    }
    if (car->unk176 != tPoints[car->unk172].type) {
        spd = GetMinSpeedNeeded(tPoints[car->unk170].type, tPoints[car->unk172].type);
        if (car->motion.vel.vy < spd) {
            car->skid[4] = 1;
            if (car->stats.unkA8 < spd) {
                car->skid[8] = 1;
            }
        }
        return;
    }
    half = car->stats.unkBC / 2;
    if ((car->motion.vel.vy - spd) < (((dist << 5) / car->motion.vel.vy) * half)) {
        goto set52;
    }
    goto set53;
set52:
    car->skid[4] = 1;
    return;
set53:
    car->skid[5] = 1;
}

void UpdateTurnStart(AICar* car)
{
    car->unk1B4 = car->unk1B0;
    if (car->motion.vel.vy < car->stats.unkB4) {
        car->unk1B4 = (car->unk1B0 * car->motion.vel.vy) / car->stats.unkB4;
        if (car->unk1B4 < 160) {
            car->unk1B4 = 160;
        }
    }
}

void AICarInitSwerve(AICar* car)
{
    VECTOR3 d;
    s32 v;
    s32 amt;

    car->unk204 = car->bearing;
    CalcDistFromRoadCenter(car, &d, 1);
    car->unk161 = 1;
    if (car->unk204 == 0) {
        car->unk188 = d.vx - car->unk186;
        if (car->unk188 <= 0) {
            car->unk162 = 0;
        } else {
            car->unk162 = 1;
        }
    } else if (car->unk204 == 1) {
        car->unk188 = d.vx - car->unk186;
        if (car->unk188 <= 0) {
            car->unk162 = 1;
        } else {
            car->unk162 = 0;
        }
    } else if (car->unk204 == 2) {
        car->unk188 = d.vy - car->unk186;
        if (car->unk188 > 0) {
            car->unk162 = 0;
        } else if (car->unk188 < 0) {
            car->unk162 = 1;
        }
    } else {
        car->unk188 = d.vy - car->unk186;
        if (car->unk188 > 0) {
            car->unk162 = 1;
        } else if (car->unk188 < 0) {
            car->unk162 = 0;
        }
    }
    v = car->unk188;
    if (v >= 241) {
        v = 240;
    } else if (v < -240) {
        v = -240;
    }
    amt = (ABS(v) * 170) / 240;
    AICarUpdateCurrPtRange(car);
    if (car->unk162 != 0) {
        car->unk1B8 = car->unk1EC - amt;
    } else {
        car->unk1B8 = car->unk1EC + amt;
    }
    BoundAngle(&car->unk1B8);
    car->unk163 = 0;
}

void AICarUpdateSwerve(AICar* car)
{
    VECTOR3 d;
    s32 v;

    CalcDistFromRoadCenter(car, &d, 1);
    if (car->unk163 == 0) {
        if (car->unk204 == 0) {
            if (car->unk162 != 0) {
                if ((d.vx - (car->unk188 / 2)) < 0) {
                    car->unk163 = 1;
                }
            } else if ((d.vx - (car->unk188 / 2)) > 0) {
                car->unk163 = 1;
            }
        } else if (car->unk204 == 1) {
            if (car->unk162 != 0) {
                if ((d.vx - (car->unk188 / 2)) > 0) {
                    car->unk163 = 1;
                }
            } else if ((d.vx - (car->unk188 / 2)) < 0) {
                car->unk163 = 1;
            }
        } else if (car->unk204 == 2) {
            if (car->unk162 == 0) {
                if ((d.vy - (car->unk188 / 2)) < 0) {
                    car->unk163 = 1;
                }
            } else if ((d.vy - (car->unk188 / 2)) > 0) {
                car->unk163 = 1;
            }
        } else {
            if (car->unk162 != 0) {
                if ((d.vy - (car->unk188 / 2)) < 0) {
                    car->unk163 = 1;
                }
            } else if ((d.vy - (car->unk188 / 2)) > 0) {
                car->unk163 = 1;
            }
        }
        if (car->unk163 != 0) {
            car->unk1B8 = car->unk1BC;
        }
    } else {
        v = abs(car->motion.rot.vz - car->unk1BC);
        if (v < 22) {
            car->unk161 = 0;
        }
    }
    car->unk163 = 1;
    car->unk161 = 0;
    if (car->heliFlag != 0) {
        if (GetBeadOnPlayer(car, car->unk1E4)) {
            car->motion.rot.vz = SmoothAngleValue(car->motion.rot.vz, car->unk30, 0x5A);
        } else {
            car->heliFlag = 0;
        }
    } else if (car->unk161 != 0) {
        car->motion.rot.vz = SmoothAngleValue(car->motion.rot.vz, car->unk1B8, 0x5A);
    } else {
        car->motion.rot.vz = SmoothAngleValue(car->motion.rot.vz, car->unk1EC, 0x5A);
    }
}

u8 GetBeadOnPlayer(AICar* car, s32 range)
{
    PlayerCar* info;
    s16 pt;
    s32 d;
    u8 result;
    u16 ppt;

    pt = car->unk176;
    info = GetPlayerInfo(car->unk0E);
    ppt = info->unkC0;
    d = abs(car->motion.pos.vz - info->motion.pos.vz);
    result = 0;
    if (pt == (s16)ppt) {
        if (car->unk18A >= 81 && (s16)car->stats.unk22 < 6 && d < 200) {
            if (uaUsingLanes()) {
                if (car->unk01 == 0) {
                    result = car->unk34 < range;
                }
            } else if (car->unk01 == 0 && car->unk34 < range * 2) {
                result = 1;
            }
        }
    }
    if (car->uaIndex == UA_NONE) {
        result = 1;
    }
    return result;
}

s32 GetBufferZoneSpeed(AICar* car)
{
    PlayerCar* info;
    s32 diff;
    s32 t;
    s32 spd;
    s32 result;
    info = GetPlayerInfo(car->unk0E);
    diff = car->motion.rot.vz - info->motion.rot.vz;
    BoundAngle(&diff);
    if (car->unk48 == 0) {
        t = diff;
        if (t < 0)
            t = -t;
        if (t < 1024 && info->motion.vel.vy > 0) {
            result = info->motion.vel.vy;
        } else if (info->motion.vel.vy >= 0) {
            spd = abs(info->motion.vel.vy);
            if (spd < (GetFieldsLastFrame() * 190) / 100) {
                result = (GetFieldsLastFrame() * 190) / 100;
            } else {
                result = car->stats.unkB4;
            }
        } else {
            result = car->stats.unkB4;
        }
    } else {
        result = car->stats.unkB4;
    }
    return result;
}

void AICarUpdateControlPad(AICar* car)
{
    car->skid[3] = car->skid[3] != 0 && car->flags[0x0B] == 0;
    car->skid[2] = car->skid[2] != 0 && car->flags[0x0B] == 0;
    car->skid[5] = car->skid[5] != 0 || car->flags[0x10] != 0;
    car->skid[4] = car->skid[4] != 0 && car->flags[0x10] == 0 && car->flags[0x17] == 0;
    car->skid[1] = car->skid[1] != 0 && car->flags[0x10] == 0 && car->flags[0x17] == 0;
}

void AICarOutOfBattle(AICar* car)
{
    s16 i;
    s16 j;

    for (i = 0; i < numAICarsInBattle; i++) {
        if (aiCarsInBattle[i] == (s16)car->playerIdx) {
            for (j = 0; j < numAICarsInBattle; j++) {
                aiCarsInBattle[i] = aiCarsInBattle[j];
            }
            numAICarsInBattle--;
            break;
        }
    }
    car->unk41 = 0;
    UpdateAICarsInBattleLaneDist();
    arena_gun_sound(1 + car->playerIdx, 0);

    car->unk42 = 0;
    car->unk43 = 0;
    car->unk44 = 0;
    car->unk45 = 0;
    car->unk46 = 0;
}

void AICarInBattle(AICar* car)
{
    s16 i;
    u8 found;
    u16 id;

    found = 0;
    if (numAICarsInBattle < 2) {
        for (i = 0; i < numAICarsInBattle; i++) {
            if (aiCarsInBattle[i] == (s16)car->playerIdx) {
                found = 1;
                break;
            }
        }
        if (found == 0) {
            id = car->playerIdx;
            aiCarsInBattle[numAICarsInBattle] = id;
            numAICarsInBattle++;
            car->unk41 = 1;
            UpdateAICarsInBattleLaneDist();
        }
    } else {
        AICarOutOfBattle(car);
        if (car->unk01 == 1 && car->direction == DIRECTION_FRONT) {
            AIInitTurnFlags(car);
            GetNextTriggerPoint(car, car->unk170);
            AICarInitTransition(car);
            CalcTurnStart(car, 0);
        }
    }
}

void UpdateAICarsInBattleLaneDist(void)
{
    s16 i;
    AICar* car;

    for (i = 0; i < numAICarsInBattle; i++) {
        car = GetAICarInfo(aiCarsInBattle[i]);
        if (uaUsingLanes()) {
            if (i == 0) {
                if (car->unk18A >= 121) {
                    car->unk184 = 56;
                    car->unk16B = 1;
                } else {
                    car->unk184 = 0;
                    car->unk16B = 0;
                }
                car->unk16A = 0;
            } else if (i == 1) {
                if (car->unk18A >= 121) {
                    car->unk184 = -56;
                    car->unk16A = 1;
                } else {
                    car->unk184 = 0;
                    car->unk16A = 0;
                }
                car->unk16B = 0;
            } else {
                car->unk184 = 0;
                car->unk16B = 0;
                car->unk16A = 0;
            }
        } else {
            car->unk184 = 0;
        }
    }
}

void AICarUpdatePlayerDir(AICar* car)
{
    PlayerCar* info;
    VECTOR delta;
    s32 ang;

    info = GetPlayerInfo(car->unk0E);
    if (car->flags[0x0B] == 0) {
        delta.vx = info->motion.pos.vx - car->motion.pos.vx;
        delta.vy = info->motion.pos.vy - car->motion.pos.vy;
        if (delta.vy == 0) {
            delta.vy = 1;
        }
        car->unk30 = ratan2(delta.vx, delta.vy);
        BoundAngle(&car->unk30);
    }
    car->unk2C = car->unk30 - car->motion.rot.vz;
    BoundAngle(&car->unk2C);
    ang = car->unk2C;
    if (ang < 0) {
        ang = -ang;
    }
    if (ang >= 1537) {
        car->direction = DIRECTION_REAR;
    } else if (car->unk2C >= 513) {
        car->direction = DIRECTION_RIGHT;
    } else if (car->unk2C < -512) {
        car->direction = DIRECTION_LEFT;
    } else {
        car->direction = DIRECTION_FRONT;
    }
}

void BringBackAICarInCollision(AICar* car)
{
    u8 done;

    done = 0;
    if (car->flags[0x00] != 0) {
        car->unk182 = 0x50;
        car->unk4D = 0;
    }
    if (car->unk4D != 0) {
        if (car->flags[0x10] == 0) {
            car->unk182 = car->unk182 + GetFieldsLastFrame();
            SetNoCarDrift(&car->motion, car->flags);
            car->motion.vel.vy = SmoothValue(
                car->motion.vel.vy, ((-(GetFieldsLastFrame() * 760)) / 100) << 5, 0x5F);
            AICarUpdateNextPtAngle(car);
            car->motion.rot.vz = SmoothAngleValue(car->motion.rot.vz, car->unk1C8, 0x5F);
            if (car->unk182 >= 81) {
                car->unk16D = 0;
                car->unk4D = 0;
                car->unk182 = 0;
                car->motion.vel.vy = 0;
            }
        }
    } else {
        if (car->stats.unk38 == 0) {
            car->collision.count = car->collision.count - GetFieldsLastFrame();
        }
        if ((s16)car->collision.count > 0 && car->motion.vel.vy != 0) {
            if (car->motion.vel.vy > 0) {
                car->motion.vel.vy = car->motion.vel.vy - (car->stats.unkBC / 2);
                if (car->motion.vel.vy < 0) {
                    car->motion.vel.vy = 0;
                }
            } else {
                car->motion.vel.vy += car->stats.unkBC / 2;
                if (car->motion.vel.vy > 0) {
                    car->motion.vel.vy = 0;
                }
            }
        } else {
            done = 1;
        }
        if (done != 0) {
            SetNoCarDrift(&car->motion, car->flags);
            InitNoCollision(&car->collision);
            if (car->unk4B != 0) {
                car->unk4D = 1;
            } else {
                car->unk4D = 0;
                car->unk16D = 0;
            }
            GetNewPointToDriveTo(car);
        }
    }
}

void StartLostCheck(AICar* car)
{
    car->stats.unkFC = car->stats.unk100 / 2;
    RecomputeCarDeltas(&car->stats);
    car->unk49 = 1;
}
void InitControlPad(u8* pad)
{
    pad[0x0] = 0;
    pad[0x1] = 0;
    pad[0x3] = 0;
    pad[0x2] = 0;
    pad[0x5] = 0;
    pad[0x4] = 0;
    pad[0x6] = 0;
    pad[0x7] = 0;
    pad[0x8] = 0;
    pad[0x9] = 0;
    pad[0xA] = 0;
    pad[0xB] = 0;
    pad[0xC] = 0;
    pad[0xD] = 0;
    pad[0xE] = 0;
    pad[0xF] = 1;
}
void InitNoCollision(CarCollision* c)
{
    c->unk0 = 0;
    c->unk1 = 0;
    c->unk2 = 0;
    c->unk4 = 0;
    c->count = 0;
}

s32 CalcMaxRotBeforeDrift(Car* car, u8 isPlayer)
{
    CarStats* st;
    CarMotion* m;
    u8* sk;
    u8 hit;
    u8 g;
    s32 d;
    s32 hi;
    s32 t;
    s32 ret;

    if (isPlayer) {
        st = &PLAYER_CAR(car)->stats;
        g = PLAYER_CAR(car)->flags[4];
        m = &PLAYER_CAR(car)->motion;
        sk = PLAYER_CAR(car)->skid;
        hit = 0;
        if (g && PLAYER_CAR(car)->flags[6]
            && (PLAYER_CAR(car)->skid[6] || PLAYER_CAR(car)->skid[7])) {
            hit = 1;
        }
        if (sk[6] || sk[7]) {
            d = st->unkD4 * st->unkE8;
        } else {
            d = st->unkD4;
        }
    } else {
        st = &AI_CAR(car)->stats;
        g = AI_CAR(car)->flags[4];
        m = &AI_CAR(car)->motion;
        sk = AI_CAR(car)->skid;
        hit = 0;
        if (g && AI_CAR(car)->flags[6] && (AI_CAR(car)->skid[6] || AI_CAR(car)->skid[7])) {
            hit = 1;
        }
        if (sk[6] || sk[7]) {
            d = st->unkD4 * st->unkE8;
        } else {
            d = st->unkD4 * 2;
        }
    }

    if (hit) {
        return st->unkCC;
    }

    if (m->vel.vy < st->unkA8) {
        hi = st->unkA8;
    } else {
        hi = m->vel.vy;
    }
    if (st->unkB0 >= hi) {
        hi = st->unkB0 + 1;
    }

    if (m->vel.vy > st->unkB0) {
        t = (m->vel.vy / 32) * d / st->unk7C;
        if (d < t) {
            t = d;
        }
        return t - (t - st->unkD0) * (abs(m->vel.vy) - st->unkB0) / (hi - st->unkB0);
    }
    return d;
}

void InitPoliceCar(CarStats* stats)
{
    stats->unk9C = 5;
    stats->unkC4 = 3;
    stats->unk88 = 0x73;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x67;
    stats->unk98 = stats->unk88 * 3 / 4;
    stats->unkC8 = 2;
    stats->unkE0 = 0x19;
    stats->unkE4 = 5;
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
    stats->unk1E = 0x28;
    stats->unk5C = 6;
    stats->unk24 = 0x320;
}

s32 GetMinSpeedNeeded(s16 group, s16 idx)
{
    s32 speed;
    s32 fields;

    speed = (s16)triggerPtGroups.minSpeed[group][idx];
    fields = GetFieldsLastFrame() * 19;
    return ((speed * fields) / 100) << 5;
}

void UpdateCarOnSlickSpot(Car* car, u8 isPlayer)
{
    CarStats* st;
    CarMotion* m;
    s32 v;
    s32 lim;

    if (isPlayer) {
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
    } else {
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
    }
    v = m->vel.vy;
    lim = st->unkA4;
    v = abs(v);
    if (lim < v) {
        SetFullCarDrift(m);
        return;
    }
    if (isPlayer) {
        BringBackDriftingCar(car, isPlayer);
    }
}

void CalcDistFromRoadCenter(AICar* car, VECTOR3* out, u8 which)
{
    s32 a;
    s32 b;
    u8 c;
    s32 h1;
    s32 h2;
    s32 p1;
    s32 p2;
    s32 q1;
    s32 q2;
    s32 d0;
    s32 d1;

    if (which) {
        a = car->unk164;
        b = car->unk165;
        c = car->unk166;
        h1 = car->unk18E;
        h2 = car->unk190;
        p1 = car->unk198;
        p2 = car->unk19C;
        q1 = car->unk1D4;
        q2 = car->unk1D8;
    } else {
        a = car->unk167;
        b = car->unk168;
        c = car->unk169;
        h1 = car->unk192;
        h2 = car->unk194;
        p1 = car->unk1A0;
        p2 = car->unk1A4;
        q1 = car->unk1DC;
        q2 = car->unk1E0;
    }
    if ((u8)a) {
        if (h2 != 0) {
            out->vx = (car->motion.pos.vx - p1) - (h1 * (car->motion.pos.vy - p2)) / h2;
        } else {
            out->vx = 0;
        }
        if (h1 != 0) {
            out->vy = (car->motion.pos.vy - p2) - (h2 * (car->motion.pos.vx - p1)) / h1;
            d0 = out->vx;
            d1 = out->vy;
            if (d0 < 0) {
                d0 = -d0;
            }
            if (d1 < 0) {
                d1 = -d1;
            }
            if (d1 < d0) {
                out->vx = 0;
            } else {
                out->vy = 0;
            }
        } else {
            out->vy = 0;
        }
    } else if ((u8)b) {
        out->vx = car->motion.pos.vx - q1;
        out->vy = 0;
    } else {
        out->vx = 0;
        out->vy = car->motion.pos.vy - q2;
    }
}

void InitTriggerPoint(AICar* car)
{
    s16 i;
    s16 next;
    s32 choice;

    choice = arena_rand() % tPoints[car->stats.triggerPt].f1A;
    GetNextTriggerPoint(car, tPoints[car->stats.triggerPt].link[choice]);
    AICarInitTransition(car);
    CalcTurnStart(car, 0);
    UpdateCurrentTriggerPt(car);
    GetNextTriggerPoint(car, car->stats.triggerPt);
    AICarInitTransition(car);
    CalcTurnStart(car, 0);
    UpdateCurrentTriggerPt(car);
    next = 0;
    for (i = 0; i < tPoints[car->stats.triggerPt].f1A; i++) {
        if (tPoints[car->stats.triggerPt].link[i] == car->stats.unk68) {
            next = i + 1;
            if (next >= tPoints[car->stats.triggerPt].f1A) {
                next = 0;
            }
            break;
        }
    }
    car->stats.unk68 = tPoints[car->stats.triggerPt].link[next];
    GetNextTriggerPoint(car, car->stats.unk68);
    AICarInitTransition(car);
    CalcTurnStart(car, 0);
    UpdateCurrentTriggerPt(car);
    car->motion.pos.vx = tPoints[car->stats.triggerPt].vx;
    car->motion.pos.vy = tPoints[car->stats.triggerPt].vy;
    car->motion.pos.vz = triggerPtGroups.pos[tPoints[car->stats.triggerPt].type].vz;
    car->motion.vel.vy = 0;
    car->flags[1] = 0;
    car->motion.rot.vz = car->unk1BC;
    InitTireGroup(car->tires, tPoints[car->stats.triggerPt].type);
}

void UpdateCurrentTriggerPt(AICar* car)
{
    s32 dx;
    s32 dz;
    u8 both;

    car->unk170 = car->unk172;
    car->unk172 = car->unk174;
    car->unk1BC = car->unk1C0;
    car->unk1F8 = car->unk1FC;
    car->unk18A = car->unk18C;
    car->unk198 = car->unk1A0;
    car->unk19C = car->unk1A4;
    car->unk1A0 = car->unk1A8;
    car->unk1A4 = car->unk1AC;
    if (tPoints[car->unk170].type == tPoints[car->unk172].type) {
        car->unk16E = 0;
    } else {
        car->unk16E = 1;
    }
    CalcTurnStart(car, 0);
    car->unk200 = car->unk1B0;
    car->unk18E = tPoints[car->unk172].vx - tPoints[car->unk170].vx;
    car->unk190 = tPoints[car->unk172].vy - tPoints[car->unk170].vy;
    dx = car->unk18E;
    if (dx < 0) {
        dx = -dx;
    }
    car->unk165 = dx < 40;
    dz = car->unk190;
    if (dz < 0) {
        dz = -dz;
    }
    car->unk166 = dz < 40;
    both = 0;
    if (car->unk165 == 0) {
        both = car->unk166 == 0;
    }
    car->unk164 = both;
    if (car->unk164) {
        car->unk1D4 = tPoints[car->unk170].vx + car->unk18E / 2;
        car->unk1D8 = tPoints[car->unk170].vy + car->unk190 / 2;
    } else if (car->unk165) {
        car->unk1D4 = tPoints[car->unk170].vx;
    } else {
        car->unk1D8 = tPoints[car->unk170].vy;
    }
    AICarInitSwerve(car);
    car->unk160 = 0;
    car->unk174 = car->unk170;
    car->unk17E = 0;
}

void CalcTurnStart(AICar* car, u8 which)
{
    s32 v;
    s32 dx;
    s32 dz;
    u8 both;

    if (which) {
        v = car->unk1C4;
        if (v < 0) {
            v = -v;
        }
        if (v < 113) {
            car->unk1B0 = 160;
            car->stats.unk94 = car->stats.unk88;
        } else {
            v = car->unk1C4;
            if (v < 0) {
                v = -v;
            }
            if (v < 455) {
                car->unk1B0 = car->stats.unk24 / 2;
                car->stats.unk94 = 4 * car->stats.unk88 / 5;
            } else {
                v = car->unk1C4;
                if (v < 0) {
                    v = -v;
                }
                if (v < 773) {
                    car->unk1B0 = 7 * car->stats.unk24 / 10;
                    car->stats.unk94 = 3 * car->stats.unk88 / 4;
                } else {
                    v = car->unk1C4;
                    if (v < 0) {
                        v = -v;
                    }
                    if (v < 1285) {
                        car->unk1B0 = car->stats.unk24;
                        car->stats.unk94 = 2 * car->stats.unk88 / 3;
                    } else {
                        if (car->unk16C != 0) {
                            car->stats.unk94 = 2 * car->stats.unk88 / 3;
                        } else {
                            car->stats.unk94 = car->stats.unk88 / 2;
                        }
                        car->unk1B0 = 10 * car->stats.unk24 / 7;
                    }
                }
            }
        }
        if (car->unk18C < 81) {
            car->stats.unk94 = car->stats.unk94 / 2;
            car->unk1B0 = car->unk1B0 / 2;
        } else if (car->unk18C < 121) {
            car->stats.unk94 = 3 * car->stats.unk94 / 4;
            car->unk1B0 = 3 * car->unk1B0 / 4;
        }
        if (car->stats.unk94 < 20) {
            car->stats.unk94 = 20;
        }
        car->stats.unkB4
            = car->stats.unk94 * 19 * GetFieldsLastFrame() / 100 * 32 * car->stats.unkFC / 100;
        if (car->unk1B0 < 160) {
            car->unk1B0 = 160;
        }
        switch (GetAICarStrength()) {
        case 60:
            car->unk1B0 = 2 * car->unk1B0 / 3;
            break;
        case 100:
            break;
        case 120:
            car->unk1B0 = 6 * car->unk1B0 / 5;
            break;
        }
    } else {
        car->unk1B0 = 10 * car->stats.unk24 / 7;
        car->unk1B0 += car->unk1B0 / 2;
        car->stats.unk94 = 2 * car->stats.unk88 / 3;
        car->stats.unkB4
            = car->stats.unk94 * 19 * GetFieldsLastFrame() / 100 * 32 * car->stats.unkFC / 100;
    }
    car->unk192 = tPoints[car->unk174].vx - tPoints[car->unk172].vx;
    car->unk194 = tPoints[car->unk174].vy - tPoints[car->unk172].vy;
    dx = car->unk192;
    if (dx < 0) {
        dx = -dx;
    }
    car->unk168 = dx < 40;
    dz = car->unk194;
    if (dz < 0) {
        dz = -dz;
    }
    car->unk169 = dz < 40;
    both = 0;
    if (car->unk168 == 0) {
        both = car->unk169 == 0;
    }
    car->unk167 = both;
    if (car->unk167) {
        car->unk1DC = tPoints[car->unk172].vx + car->unk192 / 2;
        car->unk1E0 = tPoints[car->unk172].vy + car->unk194 / 2;
    } else if (car->unk168) {
        car->unk1DC = tPoints[car->unk172].vx;
    } else {
        car->unk1E0 = tPoints[car->unk172].vy;
    }
    car->unk1B4 = car->unk1B0;
}

void GetNextTriggerPoint(AICar* car, s16 pt)
{
    s16 r;
    VECTOR delta;
    s32 a;
    s16 next;
    s16 distance;
    s16 out[4];

    if (pt != -1) {
        car->unk174 = pt;
        car->unk18C = 96;
    } else {
        if (car->unk01 == 2) {
            r = arena_rand() % tPoints[car->unk172].f1A;
            if (tPoints[car->unk172].f1A >= 2) {
                while (tPoints[car->unk172].link[r] == car->unk170) {
                    r = arena_rand() % tPoints[car->unk172].f1A;
                }
            }
            car->unk174 = tPoints[car->unk172].link[(s16)r];
            car->unk18C = tPoints[car->unk172].dist[(s16)r];
        } else {
            next = GetClosestTriggerPointFromCurrPt(car, &tPoints[car->unk172], out);
            distance = out[0];
            car->unk174 = next;
            car->unk18C = distance;
        }
    }
    delta.vx = tPoints[car->unk174].vx - tPoints[car->unk172].vx;
    delta.vy = tPoints[car->unk174].vy - tPoints[car->unk172].vy;
    car->unk1FC = arena_sqrt(delta.vx * delta.vx + delta.vy * delta.vy);
    if (delta.vy == 0) {
        delta.vy = 1;
    }
    car->unk1C0 = ratan2(delta.vx, delta.vy);
    BoundAngle(&car->unk1C0);
    if (car->unk16B != 0) {
        car->unk1A8 = tPoints[car->unk174].vx + car->unk184;
        car->unk1AC = tPoints[car->unk174].vy + car->unk184;
    } else if (car->unk16A != 0) {
        car->unk1A8 = tPoints[car->unk174].vx - car->unk184;
        car->unk1AC = tPoints[car->unk174].vy - car->unk184;
    } else {
        car->unk1A8 = tPoints[car->unk174].vx;
        car->unk1AC = tPoints[car->unk174].vy;
    }
    car->unk1C4 = car->unk1C0 - car->unk1BC;
    BoundAngle(&car->unk1C4);
    a = car->unk1C4;
    if (a < 0) {
        a = -a;
    }
    if (a >= 1593) {
        car->unk16C = 1;
    } else {
        car->unk16C = 0;
    }
}

s16 GetClosestTriggerPointFromCurrPt(AICar* car, TriggerPt* tp, s16* out)
{
    s16 la;
    s16 bestPt;
    s16 pt;
    s16 dist;
    s16 i;
    s32 best;
    s32 d;
    VECTOR3 lap;

    la = GetLookAheadPos(car, &lap);
    bestPt = tp->link[0];
    *out = tp->dist[0];
    best = 0x7FFFFFFF;
    if (car->unk01 == 1) {
        best = 0;
    }
    for (i = 0; i < tp->f1A; i++) {
        pt = tp->link[i];
        if (car->unk01 == 0 && la >= 0 && tPoints[pt].type != la) {
            continue;
        }
        d = arena_sqrt((lap.vx - tPoints[pt].vx) * (lap.vx - tPoints[pt].vx)
            + (lap.vy - tPoints[pt].vy) * (lap.vy - tPoints[pt].vy));
        dist = tp->dist[i];
        if (car->unk01 == 1) {
            if (d > best && (pt != car->unk170 || car->unk17C < 3)) {
                bestPt = pt;
                best = d;
                *out = dist;
            }
        } else {
            if (d < best && (pt != car->unk170 || (dist >= 121 && car->unk17C < 3))) {
                bestPt = pt;
                best = d;
                *out = dist;
            }
        }
    }
    if (bestPt == car->unk170) {
        car->unk17C++;
    } else {
        car->unk17C = 0;
    }
    return bestPt;
}

s16 GetLookAheadPos(AICar* car, VECTOR3* out)
{
    PlayerCar* player;
    s16 grp;
    s16 current;
    s8 mode;
    s16 type;
    s16 ret;
    s16 tp;
    s16 tgt;
    s16 link;
    s16 t;
    s16 i;
    s32 a;
    s32 b;
    VECTOR3 vin;
    VECTOR3 vout;

    player = GetPlayerInfo(car->unk0E);
    current = car->unk172;
    grp = player->unkC0;
    ret = type = tPoints[current].type;
    tp = triggerPtGroups.a[type][grp];
    tgt = triggerPtGroups.b[type][grp];
    if (grp == type || tp < 0) {
        car->unk4C = 0;
    } else {
        car->unk4C = 1;
    }
    if (car->unk4C != 0) {
        mode = car->unk01;
        if (mode == 0) {
            if (tp == car->unk172) {
                link = tPoints[tp].link[0];
                for (i = mode; i < tPoints[tp].f1A; i++) {
                    t = tPoints[tp].link[i];
                    if (tPoints[t].type == tgt) {
                        link = t;
                        ret = tgt;
                        if (GetFrameCount() & 1) {
                            break;
                        }
                    }
                }
                tp = link;
            }
        } else {
            tp = *(s16*)&player->flags[0x1E];
            ret = -1;
        }
        out->vx = tPoints[tp].vx;
        out->vy = tPoints[tp].vy;
        out->vz = tPoints[tp].f08;
    } else {
        vin.vx = 0;
        vin.vy = car->unk34 / 2;
        vin.vz = 0;
        if (car->unk01 == 0) {
            if (player->motion.vel.vy != 0) {
                a = car->stats.unk10C * player->motion.vel.vy;
                if (player->motion.vel.vy < player->stats.unkA8) {
                    b = player->stats.unkA8;
                } else {
                    b = player->motion.vel.vy;
                }
                a /= b;
                vin.vy = a;
            } else {
                vin.vy = 240;
            }
        }
        mathMulTransVec(&player->motion.mat2, &vin, &vout);
        out->vx = player->motion.pos.vx + vout.vx;
        out->vy = player->motion.pos.vy + vout.vy;
        ret = -1;
        out->vz = player->motion.pos.vz + vout.vz;
    }
    return ret;
}

void AICarInitProfiles(AICar* car)
{
    CarDrivingProfile* profile;
    car->unk104 = car->stats.unk44 / 5;
    car->unk105 = (car->stats.unk44 * 49) / 100;
    car->unk106 = (car->stats.unk44 * 75) / 100;
    car->unk107 = car->stats.unk44;
    car->profile.attack[0].unk02 = 0;
    car->profile.attack[1].unk02 = 0;
    car->profile.attack[2].unk02 = 90;
    car->profile.attack[3].unk02 = 90;
    car->profile.attack[0].unk04 = 50;
    car->profile.attack[1].unk04 = 50;
    car->profile.attack[2].unk04 = 50;
    car->profile.attack[3].unk04 = 50;
    car->profile.attack[0].unk06 = 50;
    car->profile.attack[1].unk06 = 50;
    car->profile.attack[2].unk06 = 50;
    car->profile.attack[3].unk06 = 50;
    profile = &car->profile;
    switch (car->uaIndex) {
    case UA_ICE_CREAM_TRUCK:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 0;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 50;
        profile->route[1].unk04 = 100;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 100;
        profile->route[2].unk04 = 100;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 100;
        profile->route[3].unk04 = 100;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 90;
        profile->attack[1].unk00 = 90;
        profile->attack[2].unk00 = 90;
        profile->attack[3].unk00 = 90;
        break;
    case UA_TAXI_CAB:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 100;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 10;
        profile->route[1].unk04 = 80;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 60;
        profile->route[2].unk04 = 85;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 80;
        profile->route[3].unk04 = 95;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 30;
        profile->attack[3].unk00 = 30;
        break;
    case UA_SEMI_TRUCK:
        profile->route[0].prio = 70;
        profile->route[0].unk04 = 97;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 180;
        profile->route[0].unk10 = 120;
        profile->route[0].unk14 = 60;
        profile->route[1].prio = 80;
        profile->route[1].unk04 = 90;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 180;
        profile->route[1].unk10 = 120;
        profile->route[1].unk14 = 60;
        profile->route[2].prio = 90;
        profile->route[2].unk04 = 90;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 180;
        profile->route[2].unk10 = 120;
        profile->route[2].unk14 = 60;
        profile->route[3].prio = 95;
        profile->route[3].unk04 = 95;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 180;
        profile->route[3].unk10 = 120;
        profile->route[3].unk14 = 60;
        profile->attack[0].unk00 = 90;
        profile->attack[1].unk00 = 90;
        profile->attack[2].unk00 = 90;
        profile->attack[3].unk00 = 90;
        break;
    case UA_MONSTER_TRUCK:
        profile->route[0].prio = 90;
        profile->route[0].unk04 = 100;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 50;
        profile->route[1].unk04 = 100;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 50;
        profile->route[2].unk04 = 100;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 90;
        profile->route[3].unk04 = 95;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 90;
        profile->attack[1].unk00 = 90;
        profile->attack[2].unk00 = 100;
        profile->attack[3].unk00 = 100;
        break;
    case UA_LAMBORGHINI:
        profile->route[0].prio = 10;
        profile->route[0].unk04 = 98;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 120;
        profile->route[0].unk10 = 120;
        profile->route[0].unk14 = 60;
        profile->route[1].prio = 10;
        profile->route[1].unk04 = 98;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 120;
        profile->route[1].unk10 = 120;
        profile->route[1].unk14 = 60;
        profile->route[2].prio = 10;
        profile->route[2].unk04 = 98;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 120;
        profile->route[2].unk10 = 120;
        profile->route[2].unk14 = 60;
        profile->route[3].prio = 30;
        profile->route[3].unk04 = 98;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 120;
        profile->route[3].unk10 = 120;
        profile->route[3].unk14 = 60;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 0;
        profile->attack[3].unk00 = 0;
        break;
    case UA_POLICE_CAR:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 85;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 420;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 0;
        profile->route[1].unk04 = 85;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 420;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 85;
        profile->route[2].unk04 = 85;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 600;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 85;
        profile->route[3].unk04 = 85;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 600;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 50;
        profile->attack[3].unk00 = 50;
        break;
    case UA_HUMVEE:
        profile->route[0].prio = 80;
        profile->route[0].unk04 = 98;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 120;
        profile->route[0].unk10 = 600;
        profile->route[0].unk14 = 150;
        profile->route[1].prio = 80;
        profile->route[1].unk04 = 98;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 120;
        profile->route[1].unk10 = 600;
        profile->route[1].unk14 = 150;
        profile->route[2].prio = 80;
        profile->route[2].unk04 = 98;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 120;
        profile->route[2].unk10 = 600;
        profile->route[2].unk14 = 150;
        profile->route[3].prio = 80;
        profile->route[3].unk04 = 98;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 120;
        profile->route[3].unk10 = 600;
        profile->route[3].unk14 = 150;
        profile->attack[0].unk00 = 65;
        profile->attack[1].unk00 = 65;
        profile->attack[2].unk00 = 65;
        profile->attack[3].unk00 = 65;
        break;
    case UA_HARLEY:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 0;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 600;
        profile->route[1].prio = 0;
        profile->route[1].unk04 = 0;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 600;
        profile->route[2].prio = 0;
        profile->route[2].unk04 = 0;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 600;
        profile->route[3].prio = 0;
        profile->route[3].unk04 = 0;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 600;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 0;
        profile->attack[3].unk00 = 0;
        break;
    case UA_DUNE_BUGGY:
        profile->route[0].prio = 15;
        profile->route[0].unk04 = 90;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 600;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 90;
        profile->route[1].unk04 = 95;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 480;
        profile->route[1].unk10 = 180;
        profile->route[1].unk14 = 180;
        profile->route[2].prio = 15;
        profile->route[2].unk04 = 90;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 600;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 15;
        profile->route[3].unk04 = 90;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 600;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 10;
        profile->attack[3].unk00 = 10;
        break;
    case UA_LOW_RIDER:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 0;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 100;
        profile->route[0].unk14 = 100;
        profile->route[1].prio = 0;
        profile->route[1].unk04 = 0;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 120;
        profile->route[2].prio = 0;
        profile->route[2].unk04 = 0;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 120;
        profile->route[3].prio = 65;
        profile->route[3].unk04 = 65;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 120;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 5;
        profile->attack[3].unk00 = 5;
        break;
    case UA_CORVETTE:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 100;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 95;
        profile->route[1].unk04 = 100;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 600;
        profile->route[1].unk10 = 600;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 95;
        profile->route[2].unk04 = 100;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 600;
        profile->route[2].unk10 = 600;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 95;
        profile->route[3].unk04 = 100;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 600;
        profile->route[3].unk10 = 600;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 5;
        profile->attack[3].unk00 = 5;
        break;
    case UA_MAD_MAX:
        profile->route[0].prio = 0;
        profile->route[0].unk04 = 100;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 10;
        profile->route[1].unk04 = 80;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 60;
        profile->route[2].unk04 = 85;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 80;
        profile->route[3].unk04 = 95;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 30;
        profile->attack[3].unk00 = 50;
        break;
    case UA_BOSS_CAR:
        profile->route[0].prio = 30;
        profile->route[0].unk04 = 90;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 300;
        profile->route[0].unk14 = 300;
        profile->route[1].prio = 60;
        profile->route[1].unk04 = 80;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 300;
        profile->route[1].unk14 = 300;
        profile->route[2].prio = 90;
        profile->route[2].unk04 = 95;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 300;
        profile->route[2].unk14 = 300;
        profile->route[3].prio = 90;
        profile->route[3].unk04 = 90;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 300;
        profile->route[3].unk14 = 300;
        profile->attack[0].unk00 = 100;
        profile->attack[1].unk00 = 100;
        profile->attack[2].unk00 = 100;
        profile->attack[3].unk00 = 100;
        break;
    default:
        profile->route[0].prio = 100;
        profile->route[0].unk04 = 100;
        profile->route[0].unk08 = 100;
        profile->route[0].val = 300;
        profile->route[0].unk10 = 0;
        profile->route[0].unk14 = 0;
        profile->route[1].prio = 100;
        profile->route[1].unk04 = 100;
        profile->route[1].unk08 = 100;
        profile->route[1].val = 300;
        profile->route[1].unk10 = 0;
        profile->route[1].unk14 = 0;
        profile->route[2].prio = 100;
        profile->route[2].unk04 = 100;
        profile->route[2].unk08 = 100;
        profile->route[2].val = 300;
        profile->route[2].unk10 = 0;
        profile->route[2].unk14 = 0;
        profile->route[3].prio = 100;
        profile->route[3].unk04 = 100;
        profile->route[3].unk08 = 100;
        profile->route[3].val = 300;
        profile->route[3].unk10 = 0;
        profile->route[3].unk14 = 0;
        profile->attack[0].unk00 = 0;
        profile->attack[1].unk00 = 0;
        profile->attack[2].unk00 = 0;
        profile->attack[3].unk00 = 0;
        break;
    }
}

// Single-opponent arena restores the battle list from its sole member.
void tm1_ai_restore_battle(AICar* car)
{
    numAICarsInBattle = car->unk41 != 0;
    aiCarsInBattle[0] = car->playerIdx;
}

void UpdateNumAICarsInBattle(void)
{
    s16 i;
    AICar* car;

    for (i = 0; i < numAICarsInBattle; i++) {
        car = GetAICarInfo(aiCarsInBattle[i]);
        if (car->stats.unk40 <= 0) {
            AICarOutOfBattle(car);
            return;
        }
    }
}

s16 GetNumAICarsInBattle(void)
{
    return numAICarsInBattle;
}

void tm1_load_battle(const s16* battle)
{
    numAICarsInBattle = battle[0];
    aiCarsInBattle[0] = battle[1];
    aiCarsInBattle[1] = battle[2];
}
void tm1_store_battle(s16* battle)
{
    battle[0] = numAICarsInBattle;
    battle[1] = aiCarsInBattle[0];
    battle[2] = aiCarsInBattle[1];
}

void AICarHeliUpdate(AICar* car)
{
    PlayerCar* info;
    VECTOR3 v;
    VECTOR3 out;
    VECTOR w;
    s32 speed;
    s32 t;
    s32 ang;

    info = GetPlayerInfo(car->unk0E);
    AICarUpdateClosestPlayer(car);
    AICarUpdatePlayerDir(car);
    InitControlPad(car->skid);
    RecomputeCarDeltas(&car->stats);
    if (car->unk160 != 0) {
        AICarTurn(car);
    } else {
        car->unk161 = 0;
        SetNoCarDrift(&car->motion, car->flags);
        v.vx = 0;
        v.vy = -600;
        v.vz = 0;
        mathMulTransVec(&info->motion.mat2, &v, &out);
        w.vx = out.vx + info->motion.pos.vx;
        w.vy = out.vy + info->motion.pos.vy;
        v.vx = w.vx - car->motion.pos.vx;
        v.vy = w.vy - car->motion.pos.vy;
        car->unk34 = arena_sqrt((v.vx * v.vx) + (v.vy * v.vy));
        speed = 0;
        if (car->unk34 >= 800) {
            if (car->unk34 < 1600) {
                s32 t1 = info->motion.vel.vy;
                t1 = ABS(t1);
                if (t1 < car->stats.unkAC) {
                    speed = car->stats.unkAC;
                } else {
                    speed = abs(info->motion.vel.vy);
                }
            } else {
                s32 t2 = info->motion.vel.vy;
                t2 = ABS(t2);
                if ((t2 * 2) < car->stats.unkAC) {
                    speed = car->stats.unkAC;
                } else {
                    s32 t3 = info->motion.vel.vy;
                    t3 = ABS(t3);
                    speed = t3 * 2;
                }
            }
        }
        if (speed > 0) {
            s32 cur = abs(car->motion.vel.vy);
            if (cur < ((GetFieldsLastFrame() * 380) / 100) || car->unk34 > 0x1FFFFF
                || (car->motion.vel.vy - speed)
                    < (((car->unk34 << 5) / car->motion.vel.vy) * (car->stats.unkBC / 10))) {
                car->skid[4] = 1;
            } else {
                car->skid[5] = 1;
            }
        } else {
            car->skid[5] = 1;
        }
        t = car->unk2C;
        t = ABS(t);
        if (t >= 1537) {
            if (car->motion.vel.vy > car->stats.unkAC) {
                car->skid[5] = 1;
                car->skid[4] = 0;
            }
            if (car->unk2C > 0) {
                ang = car->motion.rot.vz + 341;
            } else {
                ang = car->motion.rot.vz - 227;
            }
        } else {
            ang = SmoothAngleValue(car->motion.rot.vz, car->unk30, 0x5F);
        }
        car->motion.rot.vz = ang;
    }
    CarUpdateDeltas(car, 0);
    CarRotUpdate(car, 0);
    CarTransUpdate(car, 0);
}
