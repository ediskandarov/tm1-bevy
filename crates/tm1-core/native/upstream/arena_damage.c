#include "tm1/arena_ground.h"
#include "tm1/arena_guns.h"

void InitCatapult(Car* car, u8 isPlayer)
{
    CarTire* t;
    u8* f;
    CarMotion* m;
    s16 v;

    if (isPlayer) {
        t = PLAYER_CAR(car)->tires;
        f = PLAYER_CAR(car)->flags;
        m = &PLAYER_CAR(car)->motion;
    } else {
        t = AI_CAR(car)->tires;
        f = AI_CAR(car)->flags;
        m = &AI_CAR(car)->motion;
    }

    if (f[0xB] == 0) {
        if (t[0].unk6 || t[1].unk6 || t[2].unk6 || t[3].unk6) {
            v = 15;
        } else {
            v = 30;
        }
        t[0].unk14 = v;
        t[1].unk14 = v;
        t[2].unk14 = v;
        t[3].unk14 = v;
        if (GetFrameCount() & 1) {
            t[0].unk12 -= 2;
        } else {
            t[1].unk12 -= 2;
        }
        m->vel.vy = m->vel.vy * 3 / 2;
        t[0].unk3 = 1;
        t[1].unk3 = 1;
        t[2].unk3 = 1;
        t[3].unk3 = 1;
        t[0].catapulted = 1;
        t[1].catapulted = 1;
        t[2].catapulted = 1;
        t[3].catapulted = 1;
        t[0].unk0 = 1;
        t[1].unk0 = 1;
        t[2].unk0 = 1;
        t[3].unk0 = 1;
        f[0xF] = 1;
        f[0xB] = 1;
        f[1] = 1;
    }
}

void UpdateCatapultedTire(CarTire* t)
{
    if (!t->unk3) {
        t->catapulted = 0;
        t->unk1C = 0;
    }
}

void CheckCatapults(Car* car, u8 isPlayer)
{
    CarTire* t;
    u8* f;

    if (isPlayer) {
        t = PLAYER_CAR(car)->tires;
        f = PLAYER_CAR(car)->flags;
    } else {
        t = AI_CAR(car)->tires;
        f = AI_CAR(car)->flags;
    }

    f[0xF] = 0;
    if (t[0].catapulted) {
        f[0xF] = 1;
        UpdateCatapultedTire(&t[0]);
    }
    if (t[1].catapulted) {
        f[0xF] = 1;
        UpdateCatapultedTire(&t[1]);
    }
    if (t[2].catapulted) {
        f[0xF] = 1;
        UpdateCatapultedTire(&t[2]);
    }
    if (t[3].catapulted) {
        f[0xF] = 1;
        UpdateCatapultedTire(&t[3]);
    }
}

void InitBombDamage(Car* car, u8 isPlayer, s8 full, u8 noMotion)
{
    CarTire* t;
    CarMotion* m;
    CarStats* st;
    u8* f;
    s32* dmg;
    s32 pct;
    s32 v;

    if (isPlayer) {
        t = PLAYER_CAR(car)->tires;
        pct = PLAYER_CAR(car)->stats.unk104;
        f = PLAYER_CAR(car)->flags;
        m = &PLAYER_CAR(car)->motion;
        st = &PLAYER_CAR(car)->stats;
        dmg = &PLAYER_CAR(car)->motion.unkD4;
    } else {
        t = AI_CAR(car)->tires;
        pct = AI_CAR(car)->stats.unk104;
        f = AI_CAR(car)->flags;
        m = &AI_CAR(car)->motion;
        st = &AI_CAR(car)->stats;
        dmg = &AI_CAR(car)->motion.unkD4;
    }

    if (f[11]) {
        return;
    }

    v = (100 - pct) * 28 / 100;
    if (v < 10) {
        v = 10;
    }
    if (!full) {
        v = v / 3;
    }
    if (t[0].unk6 || t[1].unk6 || t[2].unk6 || t[3].unk6) {
        v = 10;
    }
    t[0].unk14 = v;
    t[1].unk14 = v;
    t[2].unk14 = v;
    t[3].unk14 = v;

    switch (arena_rand() & 7) {
    case 0:
        t[0].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz -= 512;
            m->vel.vy -= m->vel.vy / 4;
        }
        *dmg = -st->unkD0;
        break;
    case 1:
        t[2].unk12 -= 2;
        t[3].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz -= 113;
            m->vel.vy -= m->vel.vy / 2;
        }
        *dmg = -st->unkD0 / 4;
        break;
    case 2:
        t[1].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz += 512;
            m->vel.vy -= m->vel.vy / 4;
        }
        *dmg = st->unkD0;
        break;
    case 3:
        t[1].unk12 -= 2;
        t[3].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz += 1024;
            m->vel.vy -= m->vel.vy / 2;
        }
        *dmg = st->unkD0 * 2;
        break;
    case 4:
        t[3].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz += 1536;
            m->vel.vy -= m->vel.vy / 4;
        }
        *dmg = st->unkD0;
        break;
    case 5:
        t[2].unk12 -= 2;
        t[3].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz += 113;
        }
        *dmg = st->unkD0 / 4;
        break;
    case 6:
        t[2].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz -= 1536;
            m->vel.vy -= m->vel.vy / 4;
        }
        *dmg = -st->unkD0;
        break;
    case 7:
    default:
        t[0].unk12 -= 2;
        t[2].unk12 -= 2;
        if (!noMotion) {
            m->rot2.vz -= 1024;
            m->vel.vy -= m->vel.vy / 2;
        }
        *dmg = -(st->unkD0 * 2);
        break;
    }

    t[0].unk3 = 1;
    t[1].unk3 = 1;
    t[2].unk3 = 1;
    t[3].unk3 = 1;
    t[0].bombDamaged = 1;
    t[1].bombDamaged = 1;
    t[2].bombDamaged = 1;
    t[3].bombDamaged = 1;
    t[0].unk0 = 1;
    t[1].unk0 = 1;
    t[2].unk0 = 1;
    t[3].unk0 = 1;
    f[11] = 1;
    f[1] = 1;
    if (!full) {
        *dmg = *dmg / 3;
    }
}

void UpdateBombDamagedTire(CarTire* t, CarMotion* m)
{
    if (!t->unk3) {
        t->bombDamaged = 0;
        t->unk1E = 0;
    }
}

void CheckBombDamage(Car* car, u8 isPlayer)
{
    CarTire* t;
    u8* f;
    CarMotion* m;
    s32* p;
    u8 was;

    if (isPlayer) {
        t = PLAYER_CAR(car)->tires;
        m = &PLAYER_CAR(car)->motion;
        f = PLAYER_CAR(car)->flags;
        p = &PLAYER_CAR(car)->motion.unkD4;
    } else {
        t = AI_CAR(car)->tires;
        m = &AI_CAR(car)->motion;
        f = AI_CAR(car)->flags;
        p = &AI_CAR(car)->motion.unkD4;
    }

    was = f[0x13];
    f[0x13] = 0;
    if (t[0].bombDamaged) {
        f[0x13] = 1;
        UpdateBombDamagedTire(&t[0], m);
    }
    if (t[1].bombDamaged) {
        f[0x13] = 1;
        UpdateBombDamagedTire(&t[1], m);
    }
    if (t[2].bombDamaged) {
        f[0x13] = 1;
        UpdateBombDamagedTire(&t[2], m);
    }
    if (t[3].bombDamaged) {
        f[0x13] = 1;
        UpdateBombDamagedTire(&t[3], m);
    }
    if (f[0x13]) {
        m->rotDelta.vz += *p;
        if (was) {
            SetNoCarDrift(m, f);
        }
    }
}

void CheckSpikeDamage(Car* car, u8 isPlayer)
{
    CarStats* s;
    u8* f;

    if (isPlayer) {
        s = &PLAYER_CAR(car)->stats;
        f = PLAYER_CAR(car)->flags;
    } else {
        s = &AI_CAR(car)->stats;
        f = AI_CAR(car)->flags;
    }

    if (f[0x11] && s->spikeTimer) {
        s->spikeTimer -= GetFieldsLastFrame();
        if (s->spikeTimer <= 0) {
            f[0x11] = 0;
            s->spikeTimer = 0;
            s->unkFC = s->unk100;
            RecomputeCarDeltas(s);
        }
    }
}

void carTakeHit(s16 id, s32 amount, VECTOR3* a, u8 flag)
{
    u8 rgb[3];
    u8 isPlayer;
    u16 idx;
    u8 canHit;
    CarWeap* wp;
    s32 fields;
    s32 k;
    s32 rate;
    s32 n;
    s32 d;
    s32 mode;
    s32 dif;
    s32 ua;
    Cs* cs;
    Car* info;
    PlayerCar* car;
    AICar* alt;
    u8* f;
    CarStats* st;
    u8 enabled;

    if (amount >= 3) {
        rate = 16 / GetFieldsLastFrame();
    } else {
        rate = 6 / GetFieldsLastFrame();
    }

    if (flag) {
        rgb[0] = 0xFF;
        rgb[1] = 0xFF;
        rgb[2] = 0xFF;
    } else {
        rgb[0] = 0xFF;
        rgb[1] = 0;
        rgb[2] = 0;
    }

    if ((u8)uaIsCarMatID(id, (s8*)&isPlayer, &idx)) {
        if (isPlayer) {
            info = GetPlayerInfo(idx);
            canHit = 1;
            car = GetPlayerInfo(idx);
            st = &car->stats;
            f = car->flags;
            mode = car->unkF4;
            wp = &car->weap;
            cs = GetPlayerCs3D(idx);
            enabled = 1;
            ua = car->uaIndex;
        } else {
            info = GetAICarInfo(idx);
            alt = GetAICarInfo(idx);
            wp = &alt->weap;
            cs = GetAICs3D(idx);
            enabled = alt->unk40;
            canHit = alt->unk46;
            mode = VR_MODE_UNK_3;
            st = &alt->stats;
            f = alt->flags;
            ua = alt->uaIndex;
        }

        if (enabled && !f[0x16] && !uaHasPlayerBeatThisLevel()) {
            if (mode == VR_MODE_UNK_1) {
                SetFXSheet(rate, rgb[0], rgb[1], rgb[2]);
            } else if (amount != HIT_FREEZE && !f[0x10]) {
                UAeffectActHitLights(cs, rate, rgb);
            }

            if (amount >= 0) {
                if (amount >= 8 && (arena_rand() & 7) < amount - 7 && !f[0x1C]) {
                    InitBombDamage(info, isPlayer, 1, flag);
                }
                if (st->lostTimer) {
                    amount = amount / 2;
                }
                n = arena_rand();
                d = amount / 3;
                k = n % ((d < 2 ? 2 : d) < 8 ? (d < 2 ? 2 : d) : 8);
                explodeCreateFragments(k, &cs->pos);
                if (st->unk4C || !isPlayer) {
                    if (isPlayer) {
                        dif = uaGetDifficulty();
                        if (dif == DIFFICULTY_EASY) {
                            amount = (amount * 2) / 3;
                        }
                        if (dif == DIFFICULTY_HARD) {
                            amount = (amount * 4) / 3;
                        }
                        if (amount <= 0) {
                            amount = 1;
                        }
                    }
                    st->unk40 = st->unk40 - amount;
                }
                if (f[0x10] && amount >= 6 && !f[0]) {
                    st->unk18 = 0;
                }
            } else {
                switch (amount) {
                case HIT_FREEZE:
                    if (!f[0x10]) {
                        f[0x10] = 1;
                        st->unk18 = 0xB4;
                        UAeffectSetFreezeLight(cs);
                    } else {
                        st->unk18 = 0xB4;
                    }
                    break;
                case HIT_OIL:
                    if (ua != UA_BOSS_CAR && canHit) {
                        f[0x12] = 1;
                        f[0x0D] = 1;
                        st->unk1C = 0xC8;
                    }
                    break;
                case HIT_SPIKE:
                    f[0x11] = 1;
                    st->spikeTimer = 0xC8;
                    st->unkFC = st->unkFC / 2;
                    RecomputeCarDeltas(st);
                    break;
                case HIT_CATAPULT:
                    if (ua != UA_BOSS_CAR) {
                        InitCatapult(info, isPlayer);
                        break;
                    }

                case HIT_BOMB:
                    InitBombDamage(info, isPlayer, 1, flag);
                    break;
                case HIT_UNK_6:
                    wp->unk42 = 1;
                    break;
                }
            }
        }
    }
}

// UA.c StartDeathSequence. The frontend handles round completion.
void tm1_start_death(Car* car, u8 player)
{
    CarStats* stats = player ? &PLAYER_CAR(car)->stats : &AI_CAR(car)->stats;
    u8* flags = player ? PLAYER_CAR(car)->flags : AI_CAR(car)->flags;
    if (stats->unk40 > 0 || flags[0])
        return;
    CarMotion* motion = player ? &PLAYER_CAR(car)->motion : &AI_CAR(car)->motion;
    if (player)
        arena_gun_sound(PLAYER_CAR(car)->playerIdx, 0, &motion->pos);
    flags[0] = 1;
    flags[21] = player || AI_CAR(car)->unk2C <= 0;
    if (player && uaHasPlayerBeatThisLevel())
        return;
    VECTOR3 pos = motion->pos;
    pos.vz = 0;
    do_smoke(&pos);
    InitBombDamage(car, player, 1, 0);
    flags[20] = 1;
    stats->deathTimer = 120;
    flags[16] = 1;
    stats->unk18 = 300;
    stats->monster = 0;
    SetBounce(car, 0, 2, 0, player);
    arena_effect(&motion->pos, 29);
    arena_effect(&motion->pos, 37);
    flags[26] = 1;
}

void UpdateCarDeath(Car* car, u8 isPlayer)
{
    CarStats* st;
    CarMotion* m;
    u8* f;
    CarTire* t;
    s32 limit;
    VECTOR3 pos;
    s32 value;

    if (isPlayer) {
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
        f = PLAYER_CAR(car)->flags;
        t = PLAYER_CAR(car)->tires;
    } else {
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
        f = AI_CAR(car)->flags;
        t = AI_CAR(car)->tires;
    }

    if (st->deathTimer > 0) {
        st->deathTimer -= GetFieldsLastFrame();
        value = (st->deathTimer % 40);
        if (value == 1) {
            pos.vx = m->pos.vx;
            pos.vy = m->pos.vy;
            pos.vz = m->pos.vz;
            pos.vz = pos.vz - (st->unk6C / 2);
            do_smoke(&pos);
        }
        if (f[21]) {
            if (m->rot.vy >= -1023) {
                s32 rollRate;
                rollRate = st->unkD0 * m->rot.vy / -1024;
                value = st->unkCC;
                value += rollRate;
                m->rotDelta.vy -= value;
                m->vel.vy /= 2;
            } else {
                m->rotDelta.vy = 0;
            }
        } else if (m->rot.vy < 1024) {
            s32 rollRate;
            rollRate = st->unkD0 * m->rot.vy / 1024;
            value = st->unkCC;
            value += rollRate;
            m->rotDelta.vy += value;
            m->vel.vy /= 2;
        } else {
            m->rotDelta.vy = 0;
        }
        if (f[20]) {
            if (m->vel.vy < st->unkA4) {
                pos.vx = m->pos.vx;
                pos.vy = m->pos.vy;
                pos.vz = m->pos.vz;
                pos.vz = t->unk24;
                arena_effect(&pos, 27);
                f[20] = 0;
            }
        } else {
            limit = 3;
            if (shellGetCurrentLevel() == LEVEL6_ROOFTOPS && m->pos.vz < 400) {
                limit = 7;
            }
            if (st->blasts < limit && GetFrameCount() % 4 == 0 && (arena_rand() & 1)) {
                st->blasts++;
                f[26] = 1;
                pos = m->pos;
                if (limit == 7) {
                    pos.vx += (arena_rand() & 63) - 32;
                    pos.vy += (arena_rand() & 63) - 32;
                    pos.vz += (arena_rand() & 15) - 8;
                    arena_effect(&pos, (arena_rand() & 1) ? 29 : 30);
                } else {
                    // CAR.c samples three offsets, then overwrites the position.
                    (void)arena_rand();
                    (void)arena_rand();
                    (void)arena_rand();
                    arena_effect(&pos, (arena_rand() & 1) ? 28 : 29);
                }
            }
        }
    } else {
        m->rotDelta.vx = 0;
        m->rotDelta.vy = 0;
        m->rotDelta.vz = 0;
        m->vel.vx = 0;
        m->vel.vy -= m->vel.vy / 16;
        m->vel.vz = 0;

        if (m->rot.vy >= 1138) {
            if (m->rot.vy >= 1934) {
                return;
            }
            if (m->rot.vy < 1536) {
                m->rot.vy = SmoothAngleValue(m->rot.vy, 1024, 98);
            } else {
                m->rot.vy = SmoothAngleValue(m->rot.vy, 2036, 98);
            }
        } else if (m->rot.vy >= 114) {
            if (m->rot.vy >= 910) {
                return;
            }
            if (m->rot.vy < 512) {
                m->rot.vy = SmoothAngleValue(m->rot.vy, 0, 98);
            } else {
                m->rot.vy = SmoothAngleValue(m->rot.vy, 1024, 98);
            }
        } else if (m->rot.vy < -113) {
            if (m->rot.vy < -909) {
                return;
            }
            if (m->rot.vy >= -511) {
                m->rot.vy = SmoothAngleValue(m->rot.vy, 0, 98);
            } else {
                m->rot.vy = SmoothAngleValue(m->rot.vy, 1024, 98);
            }
        } else if (m->rot.vy < -1137) {
            if (m->rot.vy < -1933) {
                return;
            }
            if (m->rot.vy >= -1535) {
                m->rot.vy = SmoothAngleValue(m->rot.vy, -1536, 98);
            } else {
                m->rot.vy = SmoothAngleValue(m->rot.vy, -2036, 98);
            }
        } else {
            return;
        }
    }
}
