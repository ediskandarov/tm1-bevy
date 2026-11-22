#include "tm1/car_host.h"

void BoundVector(VECTOR3* v)
{
    while (v->vx >= 2048) {
        v->vx = v->vx - 4096;
    }
    while (v->vx < -2048) {
        v->vx = v->vx + 4096;
    }
    while (v->vy >= 2048) {
        v->vy = v->vy - 4096;
    }
    while (v->vy < -2048) {
        v->vy = v->vy + 4096;
    }
    while (v->vz >= 2048) {
        v->vz = v->vz - 4096;
    }
    while (v->vz < -2048) {
        v->vz = v->vz + 4096;
    }
}

void BoundAngle(s32* a)
{
    while (a[0] >= 2048) {
        a[0] = a[0] - 4096;
    }
    while (a[0] < -2048) {
        a[0] = a[0] + 4096;
    }
}

void BoundSAngle(s16* a)
{
    while (a[0] >= 2048) {
        a[0] = a[0] - 4096;
    }
    while (a[0] < -2048) {
        a[0] = a[0] + 4096;
    }
}

s32 SmoothAngleValue(s32 from, s32 to, s32 pct)
{
    s32 r;

    if (from < -1024 && to > 1024) {
        to = to - 4096;
        r = (from * pct + to * (100 - pct)) / 100;
        BoundAngle(&r);
    } else if (from > 1024 && to < -1024) {
        to = to + 4096;
        r = (from * pct + to * (100 - pct)) / 100;
        BoundAngle(&r);
    } else {
        r = (from * pct + to * (100 - pct)) / 100;
    }
    return r;
}

void InitIceCreamTruck(CarStats* stats)
{
    stats->unk9C = 5;
    stats->unkC4 = 2;
    stats->unk88 = 0x5A;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x14;
    stats->unk98 = stats->unk88 / 2;
    stats->unkC8 = 2;
    stats->unkE0 = 0x4B;
    stats->unkE4 = 3;
    stats->unk6C = 0x20;
    stats->unk70 = 0x46;
    stats->unk74 = 0x40;
    stats->unk7C = 0x46;
    stats->unk80 = 0x38;
    stats->unk84 = -0x50;
    stats->unk110 = -0x24;
    stats->unk112 = 0x4B;
    stats->unk114 = 8;
    stats->dropPower = 0;
    stats->unk11A = 0x20;
    stats->unk11C = 0x2C;
    stats->unk108 = 2;
    stats->unk04 = 3;
    stats->unk44 = 0x73;
    stats->unk50 = 0x32;
    stats->unk104 = 0x1E;
    stats->unk1E = 0x5A;
    stats->unk5C = 7;
    stats->unk24 = 0x258;
}

void InitSemiTruck(CarStats* stats)
{
    stats->unk9C = 4;
    stats->unkC4 = 5;
    stats->unk88 = 0x5A;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x1B;
    stats->unk98 = stats->unk88 / 2;
    stats->unkC8 = 3;
    stats->unkE0 = 0xF;
    stats->unkE4 = 0xA;
    stats->unkE8 = 2;
    stats->unk6C = 0x20;
    stats->unk70 = 0x37;
    stats->unk74 = 0x40;
    stats->unk7C = 0x6E;
    stats->unk80 = 0x64;
    stats->unk84 = -0x64;
    stats->unk110 = -0x13;
    stats->unk112 = 0x64;
    stats->unk114 = 0xA;
    stats->dropPower = 0;
    stats->unk11A = 0x24;
    stats->unk11C = 0x3A;
    stats->unk10C = 0x960;
    stats->unk108 = 3;
    stats->unk04 = 3;
    stats->unk44 = 0x8C;
    stats->unk50 = 0x32;
    stats->unk104 = 0x5A;
    stats->unk1E = 0x28;
    stats->unk5C = 6;
    stats->unk24 = 0x258;
}

void InitMonsterTruck(CarStats* stats)
{
    stats->unk9C = 0xA;
    stats->unkC4 = 2;
    stats->unk88 = 0x5A;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x19;
    stats->unk98 = stats->unk88 / 2;
    stats->unkC8 = 2;
    stats->unkE0 = 0x23;
    stats->unkE4 = 4;
    stats->unkE8 = 2;
    stats->unk6C = 0x42;
    stats->unk70 = 0x1B;
    stats->unk74 = 0x56;
    stats->unk7C = 0x52;
    stats->unk80 = 0x40;
    stats->unk84 = -0x50;
    stats->unk110 = 0x23;
    stats->unk112 = 0x1F;
    stats->unk114 = -0x14;
    stats->dropPower = 0;
    stats->unk11A = 4;
    stats->unk11C = 0x27;
    stats->unk10C = 0x640;
    stats->unk108 = 3;
    stats->unk04 = 3;
    stats->unk44 = 0x78;
    stats->unk50 = 0x32;
    stats->unk104 = 0x46;
    stats->unk1E = 0x5A;
    stats->unk5C = 0xA;
    stats->unk24 = 0x258;
    stats->monster = 1;
}

void InitHumvee(CarStats* stats)
{
    stats->unk9C = 6;
    stats->unkC4 = 3;
    stats->unk88 = 0x5B;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x6E;
    stats->unk98 = stats->unk88 / 2;
    stats->unkC8 = 2;
    stats->unkE0 = 0xA;
    stats->unkE4 = 4;
    stats->unkE8 = 2;
    stats->unk6C = 0x20;
    stats->unk70 = 0x26;
    stats->unk74 = 0x46;
    stats->unk7C = 0x3F;
    stats->unk80 = 0x47;
    stats->unk84 = -0x4F;
    stats->unk110 = -0x1F;
    stats->unk112 = 0x4A;
    stats->unk114 = 0xA;
    stats->dropPower = 0;
    stats->unk11A = 0xC;
    stats->unk11C = 0x24;
    stats->unk10C = 0x7D0;
    stats->unk108 = 3;
    stats->unk04 = 3;
    stats->unk44 = 0x78;
    stats->unk50 = 0x32;
    stats->unk104 = 0x50;
    stats->unk1E = 0x3C;
    stats->unk5C = 0xF;
    stats->unk24 = 0x230;
}

void InitCorvette(CarStats* stats)
{
    stats->unk9C = 0xA;
    stats->unkC4 = 3;
    stats->unk88 = 0x82;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x6E;
    stats->unk98 = stats->unk88 * 3 / 4;
    stats->unkC8 = 2;
    stats->unkE0 = 0xF;
    stats->unkE4 = 3;
    stats->unk6C = 0x10;
    stats->unk70 = 0x11;
    stats->unk74 = 0x34;
    stats->unk7C = 0x42;
    stats->unk80 = 0x30;
    stats->unk84 = -0x3E;
    stats->unk110 = -0x17;
    stats->unk112 = 0x31;
    stats->unk114 = 4;
    stats->dropPower = 0;
    stats->unk11A = -0xE;
    stats->unk11C = 0x1E;
    stats->unk10C = -0x190;
    stats->unk108 = 1;
    stats->unk04 = 1;
    stats->unk44 = 0x64;
    stats->unk50 = 0x32;
    stats->unk104 = 0x14;
    stats->unk1E = 0xF;
    stats->unk5C = 0xD;
    stats->unk24 = 0x348;
}

void InitMadMax(CarStats* stats)
{
    stats->unk9C = 6;
    stats->unkC4 = 3;
    stats->unk88 = 0x78;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x4B;
    stats->unk98 = stats->unk88 / 2;
    stats->unkC8 = 2;
    stats->unkE0 = 0x14;
    stats->unkE4 = 7;
    stats->unk6C = 0x10;
    stats->unk70 = 0x13;
    stats->unk74 = 0x34;
    stats->unk7C = 0x42;
    stats->unk80 = 0x34;
    stats->unk84 = -0x40;
    stats->unk110 = -0x18;
    stats->unk112 = 0x13;
    stats->unk114 = 4;
    stats->dropPower = 0;
    stats->unk11A = -0xE;
    stats->unk11C = 0x26;
    stats->unk10C = 0x320;
    stats->unk108 = 2;
    stats->unk04 = 2;
    stats->unk44 = 0x6E;
    stats->unk50 = 0x32;
    stats->unk104 = 0x32;
    stats->unk1E = 0x19;
    stats->unk5C = 0xA;
    stats->unk24 = 0x2F8;
}

void InitLowRider(CarStats* stats)
{
    stats->unk9C = 5;
    stats->unkC4 = 5;
    stats->unk88 = 0x78;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x55;
    stats->unk98 = stats->unk88 * 3 / 4;
    stats->unkC8 = 2;
    stats->unkE0 = 0x14;
    stats->unkE4 = 8;
    stats->unk6C = 0xE;
    stats->unk70 = 0x12;
    stats->unk74 = 0x34;
    stats->unk7C = 0x50;
    stats->unk80 = 0x3B;
    stats->unk84 = -0x4B;
    stats->unk110 = -0x1B;
    stats->unk112 = 0x3C;
    stats->unk114 = 8;
    stats->dropPower = 0;
    stats->unk11A = 3;
    stats->unk11C = 0x1E;
    stats->unk10C = 0x320;
    stats->unk108 = 2;
    stats->unk04 = 2;
    stats->unk44 = 0x6E;
    stats->unk50 = 0x32;
    stats->unk104 = 0x28;
    stats->unk1E = 0x6E;
    stats->unk5C = 6;
    stats->unk24 = 0x320;
}

void InitLamborghini(CarStats* stats)
{
    stats->unk9C = 0xD;
    stats->unkC4 = 4;
    stats->unk88 = 0x87;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x84;
    stats->unk98 = stats->unk88 * 3 / 4;
    stats->unkC8 = 2;
    stats->unkE0 = 0x14;
    stats->unkE4 = 6;
    stats->unk6C = 0x10;
    stats->unk70 = 0xF;
    stats->unk74 = 0x36;
    stats->unk7C = 0x48;
    stats->unk80 = 0x38;
    stats->unk84 = -0x3D;
    stats->unk110 = -0x1C;
    stats->unk112 = 0x2E;
    stats->unk114 = 4;
    stats->dropPower = 0;
    stats->unk11A = 4;
    stats->unk11C = 0x1E;
    stats->unk10C = -0x230;
    stats->unk108 = 1;
    stats->unk04 = 1;
    stats->unk44 = 0x64;
    stats->unk50 = 0x32;
    stats->unk104 = 0x14;
    stats->unk1E = 0xF;
    stats->unk5C = 0x14;
    stats->unk24 = 0x370;
}

void InitHarley(CarStats* stats)
{
    stats->unk9C = 7;
    stats->unkC4 = 5;
    stats->unk88 = 0x7D;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x6E;
    stats->unk98 = stats->unk88 * 7 / 8;
    stats->unkC8 = 2;
    stats->unkE0 = -0x50;
    stats->unkE4 = 5;
    stats->unk6C = 0x10;
    stats->unk70 = 7;
    stats->unk74 = 0x32;
    stats->unk7C = 0x35;
    stats->unk80 = 0x2A;
    stats->unk84 = -0x1D;
    stats->unk110 = -0x11;
    stats->unk112 = 0x12;
    stats->unk114 = 0xC;
    stats->dropPower = 0;
    stats->unk11A = -0xA;
    stats->unk11C = 0x46;
    stats->unk10C = -0x320;
    stats->unk108 = 1;
    stats->unk04 = 1;
    stats->unk44 = 0x64;
    stats->unk50 = 0x32;
    stats->unk104 = 5;
    stats->unk1E = 0x4B;
    stats->unk5C = 0xF;
    stats->unk24 = 0x2D0;
}

void InitDuneBuggy(CarStats* stats)
{
    stats->unk9C = 5;
    stats->unkC4 = 3;
    stats->unk88 = 0x6E;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x37;
    stats->unk98 = stats->unk88 * 3 / 4;
    stats->unkC8 = 2;
    stats->unkE0 = 0x1E;
    stats->unkE4 = 8;
    stats->unk6C = 0x14;
    stats->unk70 = 0x19;
    stats->unk74 = 0x40;
    stats->unk7C = 0x3A;
    stats->unk80 = 0x37;
    stats->unk84 = -0x21;
    stats->unk110 = -0xE;
    stats->unk112 = 0x29;
    stats->unk114 = 8;
    stats->dropPower = 0;
    stats->unk11A = 4;
    stats->unk11C = 0x21;
    stats->unk10C = -0x320;
    stats->unk108 = 1;
    stats->unk04 = 1;
    stats->unk44 = 0x64;
    stats->unk50 = 0x32;
    stats->unk104 = 0xA;
    stats->unk1E = 0x3C;
    stats->unk5C = 5;
    stats->unk24 = 0x280;
}

void InitTaxiCab(CarStats* stats)
{
    stats->unk9C = 0xA;
    stats->unkC4 = 3;
    stats->unk88 = 0x6E;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x1E;
    stats->unk98 = stats->unk88 * 3 / 4;
    stats->unkC8 = 2;
    stats->unkE0 = 0x14;
    stats->unkE4 = 7;
    stats->unk6C = 0x10;
    stats->unk70 = 0x1E;
    stats->unk74 = 0x30;
    stats->unk7C = 0x4E;
    stats->unk80 = 0x36;
    stats->unk84 = -0x4E;
    stats->unk110 = -0x19;
    stats->unk112 = 0x3D;
    stats->unk114 = 0xB;
    stats->dropPower = 0;
    stats->unk11A = 0;
    stats->unk11C = 0x2A;
    stats->unk10C = 0x230;
    stats->unk108 = 2;
    stats->unk04 = 2;
    stats->unk44 = 0x64;
    stats->unk50 = 0x32;
    stats->unk104 = 0x1E;
    stats->unk1E = 0x28;
    stats->unk5C = 7;
    stats->unk24 = 0x2D0;
}

void RecomputeCarDeltas(CarStats* stats)
{
    s16 rate;
    s32 k;
    s32 strengthScale;

    stats->unkA0 = stats->unk9C;
    strengthScale = stats->unkFC * 6;
    stats->unkA4 = stats->unkA0 * strengthScale / 100;
    stats->unkBC = stats->unkA0 * stats->unkC4;
    if (stats->unk00 == 0) {
        stats->unkA4 = stats->unkA4 / 2;
    }
    stats->unkCC = stats->unkC8;
    stats->unkD0 = stats->unkCC * stats->unkE4 * stats->unkFC / 100;
    rate = GetUpdateRate();
    if (rate != 0 && rate != 60) {
        stats->unkA0 = stats->unkA0 * 60 / rate;
        stats->unkA4 = stats->unkA4 * 60 / rate;
        stats->unkBC = stats->unkBC * 60 / rate;
        stats->unkCC = stats->unkCC * 60 / rate;
        stats->unkD0 = stats->unkD0 * 60 / rate;
    }
    stats->unkAC = stats->unkA4 * 3;
    stats->unkEC = stats->unkD0 * 5;
    stats->unkF0 = stats->unkD0 * 2;
    stats->unkF4 = stats->unkE0 * 682 / 100;
    stats->unkD4 = stats->unkCC * stats->unkDC;
    if (stats->unk00 == 0) {
        stats->unkD4 = stats->unkD4 * 5;
    }
    stats->unkD8 = stats->unkD4 / 2;
    k = GetFieldsLastFrame() * 19;
    stats->unkA8 = stats->unk88 * k / 100 * (stats->unkFC << 5) / 100;
    k = GetFieldsLastFrame() * 19;
    stats->unkB8 = stats->unk8C * k / 100 * (stats->unkFC << 5) / 100;
    k = GetFieldsLastFrame() * 19;
    stats->unkB0 = stats->unk90 * k / 100 * (stats->unkFC << 5) / 100;
    k = GetFieldsLastFrame() * 19;
    stats->unkB4 = stats->unk94 * k / 100 * (stats->unkFC << 5) / 100;
    k = GetFieldsLastFrame() * 19;
    stats->unkC0 = stats->unk98 * k / 100 * (stats->unkFC << 5) / 100;
}

void UpdateBearing(Car* car, u8 isPlayer)
{
    s32 ang;
    s32 t;
    CarMotion* m;
    s32* bearing;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        bearing = &PLAYER_CAR(car)->bearing;
    } else {
        m = &AI_CAR(car)->motion;
        bearing = &AI_CAR(car)->bearing;
    }

    if (m->vel.vy > 0) {
        ang = m->rot.vz;
    } else {
        ang = m->rot.vz + 2048;
        BoundAngle(&ang);
    }

    t = abs(ang);
    if (t < 512) {
        *bearing = 0;
    } else {
        s32 value;

        if (t >= 1537) {
            value = 1;
        } else {
            value = 3;
            if (ang > 0) {
                value = 2;
            }
        }
        *bearing = value;
    }
}

void CarUpdateDeltas(Car* car, u8 isPlayer)
{
    u8* input;
    u8* flags;
    CarStats* st;
    CarMotion* m;
    CarWeap* weap;
    AICar* alt;
    s32 maxTurn;
    s32 maxSpeed;
    s32 acceleration;
    s32 amount;
    s32 threshold;

    if (isPlayer) {
        input = PLAYER_CAR(car)->skid;
        flags = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
        weap = &PLAYER_CAR(car)->weap;
    } else {
        alt = AI_CAR(car);
        input = alt->skid;
        flags = alt->flags;
        st = &alt->stats;
        m = &alt->motion;
        weap = &alt->weap;
    }
    maxTurn = input[6] || input[7] ? (st->unkD4 * st->unkE8) : st->unkD4;
    maxSpeed = m->vel.vy < st->unkA8 ? st->unkA8 : m->vel.vy;
    acceleration = st->unkA4;
    if (flags[16]) {
        input[2] = 0;
        input[3] = 0;
    }
    if (input[3] || input[2]) {
        if (abs(m->vel.vy) > GetFieldsLastFrame() * 1520 / 100 * 32)
            amount = st->unkCC;
        else {
            threshold = (((GetFieldsLastFrame() * 1520) / 100) << 5);
            amount = st->unkD0 - ((m->vel.vy * (st->unkD0 - st->unkCC)) / threshold);
        }
        if ((input[3] && m->unk00 <= 0) || (input[2] && m->unk00 >= 0))
            amount = amount * 2;
        if (input[6] || input[7])
            amount = amount * st->unkE8;
        if (input[3]) {
            m->unk00 = m->unk00 + amount;
            if (maxTurn < m->unk00)
                m->unk00 = maxTurn;
        } else {
            m->unk00 = m->unk00 - amount;
            if (m->unk00 < -maxTurn)
                m->unk00 = -maxTurn;
        }
    } else if (m->unk00) {
        if (abs(m->unk00) < st->unkD0)
            m->unk00 = 0;
        else if (flags[11])
            m->unk00 -= m->unk00 / 16;
        else
            m->unk00 = m->unk00 / 2;
    }
    m->rotDelta.vz = ((m->vel.vy / 32) * (m->unk00)) / st->unk7C;
    if (flags[16]) {
        input[5] = 1;
        input[4] = 0;
        flags[5] = 0;
    } else if (isPlayer) {
        if (input[4]) {
            if (input[1]) {
                if (m->vel.vy > st->unkBC) {
                    input[4] = 0;
                    if (!input[16])
                        input[5] = 1;
                    flags[5] = 1;
                    st->unk14 = 0;
                } else {
                    flags[7] = 1;
                    input[5] = 0;
                }
            } else if (m->vel.vy < -st->unkBC) {
                input[4] = 0;
                input[5] = 1;
                flags[5] = 1;
                st->unk14 = 0;
            } else {
                flags[5] = 0;
                input[5] = 0;
            }
        } else {
            flags[5] = 0;
            input[5] = 0;
        }
    }
    if (flags[5]) {
        input[4] = 0;
        if (m->vel.vy) {
            if (abs(m->vel.vy) < st->unkA4) {
                m->vel.vy = 0;
                st->unk14 = 0;
            }
        } else {
            st->unk14 = st->unk14 + GetFieldsLastFrame();
            m->vel.vy = 0;
            if (st->unk14 >= 22)
                flags[5] = 0;
        }
    }
    if (input[8] && flags[16]) {
        if (st->unkF8 >= 91)
            goto consume_turbo;
        if (weap->unk46 > 0) {
            --weap->unk46;
            st->unkF8 += 180;
        consume_turbo:
            st->unkF8 -= 90;
            st->unk18 = 0;
        } else {
            st->unkF8 = 0;
            input[8] = 0;
        }
    }
    if (input[8]) {
        input[4] = 1;
        if (st->unkF8 <= 0 && weap->unk46 > 0) {
            --weap->unk46;
            st->unkF8 = 180;
        }
        if (st->unkF8 > 0) {
            maxSpeed = (st->unkA8 * 3) / 2;
            acceleration = st->unkA4 * 4;
            st->unkF8 = st->unkF8 - (GetFieldsLastFrame());
            if (isPlayer)
                uasoundPlayTurboBoost(0, 0);
        } else {
            maxSpeed = st->unkA8;
            acceleration = st->unkA4;
            st->unkF8 = 0;
            if (m->vel.vy > st->unkA8)
                input[4] = 0;
            if (isPlayer)
                uasoundStopTurboBoost();
        }
    } else {
        if (isPlayer)
            uasoundStopTurboBoost();
        if (m->vel.vy > st->unkA8)
            input[4] = 0;
    }
    if (input[4]) {
        flags[4] = 1;
        flags[8] = 0;
        if (((flags[7] && -st->unkB8 < m->vel.vy) || (!flags[7] && m->vel.vy < maxSpeed))
            && !flags[1]) {
            if (flags[13])
                amount = st->unkA0 / 2;
            else {
                amount = acceleration;
                if ((flags[7] && m->vel.vy < 0) || (!flags[7] && m->vel.vy > 0))
                    amount = amount - (((acceleration - st->unkA0) * abs(m->vel.vy)) / maxSpeed);
            }
            if (flags[7])
                m->vel.vy = (m->vel.vy - amount);
            else
                m->vel.vy = (m->vel.vy + amount);
        }
    } else
        flags[4] = 0;
    if (m->vel.vy && !flags[4] && !flags[8]) {
        amount = (st->unkBC - st->unkA0) * abs(m->vel.vy) / maxSpeed;
        amount /= 2;
        if (amount < st->unkA0)
            amount = st->unkA0;
        if (flags[13])
            amount /= 4;
        if (m->vel.vy > 0) {
            m->vel.vy = m->vel.vy - amount;
            if (m->vel.vy < 0)
                m->vel.vy = 0;
        } else {
            m->vel.vy = m->vel.vy + amount;
            if (m->vel.vy > 0)
                m->vel.vy = 0;
        }
    }
    if (input[1]) {
        if (m->vel.vy <= 0) {
            flags[7] = 1;
            if (-st->unkB8 < m->vel.vy) {
                flags[4] = 0;
                flags[8] = 0;
            }
        }
    } else
        flags[7] = 0;
    if (input[5] && m->vel.vy && !input[16]) {
        flags[3] = 1;
        amount = st->unkBC;
        if (flags[13])
            amount /= 4;
        if (m->vel.vy < 0) {
            m->vel.vy = m->vel.vy + amount;
            if (m->vel.vy > 0)
                m->vel.vy = 0;
        } else {
            m->vel.vy = m->vel.vy - amount;
            if (m->vel.vy < 0)
                m->vel.vy = 0;
        }
    } else
        flags[3] = 0;
}

void SetNoCarDrift(CarMotion* m, u8* f)
{
    f[1] = 0;
    m->rot2Delta.vx = m->rotDelta.vx;
    m->rot2Delta.vy = m->rotDelta.vy;
    m->rot2Delta.vz = m->rotDelta.vz;
    m->rot2.vx = m->rot.vx;
    m->rot2.vy = m->rot.vy;
    m->rot2.vz = m->rot.vz;
}

void SetFullCarDrift(CarMotion* m)
{
    m->rot2Delta.vz = 0;
}

void SlowDownNonDriftingCar(Car* car, u8 isPlayer)
{
    CarStats* st;
    CarMotion* m;
    u8 g;
    s32 d;
    s32 hi;

    if (isPlayer) {
        g = PLAYER_CAR(car)->skid[6];
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
        if (g || PLAYER_CAR(car)->skid[7]) {
            d = PLAYER_CAR(car)->stats.unkD4 * PLAYER_CAR(car)->stats.unkE8;
        } else {
            d = PLAYER_CAR(car)->stats.unkD4;
        }
    } else {
        g = AI_CAR(car)->skid[6];
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
        if (g || AI_CAR(car)->skid[7]) {
            d = AI_CAR(car)->stats.unkD4 * AI_CAR(car)->stats.unkE8;
        } else {
            d = AI_CAR(car)->stats.unkD4 * 2;
        }
    }

    if (m->vel.vy < st->unkA8) {
        hi = st->unkA8;
    } else {
        hi = m->vel.vy;
    }

    if (abs(m->vel.vy) > st->unkB0) {
        d = d - (d - st->unkEC) * abs(m->vel.vy) / hi;
        if (d < abs(m->unk00)) {
            if (m->vel.vy > 0) {
                m->vel.vy = m->vel.vy - st->unkBC;
            } else if (m->vel.vy < 0) {
                m->vel.vy = m->vel.vy + st->unkBC;
            }
        }
    }
}

void BringBackDriftingCar(Car* car, u8 isPlayer)
{
    CarStats* st;
    CarMotion* m;
    u8* f;
    u8* sk;
    s32 ang;
    s32 turn;
    s32 extra;
    s32 r;
    s32 delta;

    if (isPlayer) {
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
        f = PLAYER_CAR(car)->flags;
        sk = PLAYER_CAR(car)->skid;
    } else {
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
        f = AI_CAR(car)->flags;
        sk = AI_CAR(car)->skid;
    }

    if (abs(m->vel.vy) < st->unkAC) {
        SetNoCarDrift(m, f);
        return;
    }

    ang = m->rot.vz - m->rot2.vz;
    BoundAngle(&ang);
    if (abs(ang) >= 1366) {
        if (GetFieldsLastFrame() * 380 / 100 * 32 < m->vel.vy) {
            return;
        }
    }

    if (ang > 0) {
        if (sk[3]) {
            turn = 0;
        } else if (sk[2]) {
            turn = GetFieldsLastFrame() * 5;
        } else {
            turn = GetFieldsLastFrame() * 2;
        }
    } else {
        if (sk[3]) {
            turn = GetFieldsLastFrame() * 5;
        } else if (sk[2]) {
            turn = 0;
        } else {
            turn = GetFieldsLastFrame() * 2;
        }
    }

    if (turn > 0) {
        if (m->vel.vy > st->unkB0) {
            extra = 0;
        } else if (st->unkB0 / 2 < m->vel.vy) {
            extra = GetFieldsLastFrame() * 2;
        } else {
            extra = GetFieldsLastFrame() * 5;
        }
    } else {
        extra = 0;
    }

    delta = (extra + turn) / 2 * 4096 / 360;
    if (f[4]) {
        delta = delta * 2;
    }

    r = SmoothAngleValue(m->rot2.vz, m->rot.vz, 90);
    if (m->rot2.vz < r) {
        m->rot2.vz = m->rot2.vz + delta;
        BoundAngle(&m->rot2.vz);
        if (m->rot2.vz < m->rot.vz) {
            return;
        }
    } else {
        m->rot2.vz = m->rot2.vz - delta;
        BoundAngle(&m->rot2.vz);
        if (m->rot2.vz > m->rot.vz) {
            return;
        }
    }
    SetNoCarDrift(m, f);
}

void SlowDownDriftingCar(Car* car, u8 isPlayer)
{
    s32 ang;
    s32 t;
    s32 d;
    u8* f;
    CarStats* st;
    CarMotion* m;

    if (isPlayer) {
        f = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
    } else {
        f = AI_CAR(car)->flags;
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
    }

    ang = m->rot.vz - m->rot2.vz;
    BoundAngle(&ang);
    t = abs(ang);

    if (t < 682) {
        d = st->unkBC / 4;
    } else if (t < 1024) {
        d = st->unkBC / 2;
    } else if (t < 1365) {
        d = st->unkBC;
    } else if (f[4]) {
        d = st->unkBC * 3;
    } else {
        d = st->unkBC * 2;
    }

    if (m->vel.vy > 0) {
        m->vel.vy -= d;
        if (m->vel.vy < 0) {
            SetNoCarDrift(m, f);
        }
    } else {
        m->vel.vy += d;
        if (m->vel.vy > 0) {
            SetNoCarDrift(m, f);
        }
    }
}

void UpdateDriftingCar(PlayerCar* car)
{
    u8 doit;

    doit = 1;
    if (car->tires[0].unk3) {
        if (car->tires[1].unk3) {
            doit = 0;
        } else if (GetFrameCount() & 1) {
            doit = 0;
        }
    } else if (car->tires[1].unk3) {
        if (GetFrameCount() & 1) {
            doit = 0;
        }
    }

    if (doit) {
        if (car->skid[0x10]) {
            car->flags[1] = 1;
            SetFullCarDrift(&car->motion);
        } else if (!car->flags[0xB]) {
            BringBackDriftingCar(car, 1);
        }
        SlowDownDriftingCar(car, 1);
    }
}

void UpdateNonDriftingCar(PlayerCar* car)
{
    s32 limit;

    if (car->skid[0x10] && car->motion.vel.vy > car->stats.unkB0) {
        goto drift;
    }
    if (car->motion.vel.vy > car->stats.unkB0) {
        s32 delta;

        delta = car->motion.rotDelta.vz;
        limit = car->stats.unkD8;
        if (delta < 0) {
            delta = -delta;
        }
        limit = limit < delta;
        if (limit) {
            goto drift;
        }
    }
    if (car->skid[0x10]) {
        goto drift;
    }
    SetNoCarDrift(&car->motion, car->flags);
    goto slow;
drift:
    car->flags[1] = 1;
    SetFullCarDrift(&car->motion);
slow:
    SlowDownNonDriftingCar(car, 1);
}

void CarCheckDynamics(PlayerCar* car)
{
    if (car->flags[1]) {
        UpdateDriftingCar(car);
    } else {
        UpdateNonDriftingCar(car);
    }
}

void CarRotUpdate(Car* car, u8 isPlayer)
{
    SVECTOR rot;
    CarMotion* m;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
    } else {
        m = &AI_CAR(car)->motion;
    }

    m->rot.vx += m->rotDelta.vx;
    m->rot.vy += m->rotDelta.vy;
    m->rot.vz += m->rotDelta.vz;
    m->rot2.vx += m->rot2Delta.vx;
    m->rot2.vy += m->rot2Delta.vy;
    m->rot2.vz += m->rot2Delta.vz;
    BoundSAngle(&m->unk00);
    BoundVector(&m->rot);
    BoundVector(&m->rot2);
    rot.vx = m->rot2.vx;
    rot.vy = m->rot2.vy;
    rot.vz = m->rot2.vz;
    RotMatrixYXZ(&rot, &m->mat);
    rot.vx = m->rot.vx;
    rot.vy = m->rot.vy;
    rot.vz = m->rot.vz;
    RotMatrixYXZ(&rot, &m->mat2);
}

void CarTransUpdate(Car* car, u8 isPlayer)
{
    VECTOR3 v;
    VECTOR3 out;
    CarMotion* m;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
    } else {
        m = &AI_CAR(car)->motion;
    }

    v.vx = m->vel.vx / 32;
    v.vy = m->vel.vy / 32;
    v.vz = m->vel.vz / 32;
    mathMulTransVec(&m->mat, &v, &out);
    m->pos.vx += out.vx;
    m->pos.vy += out.vy;
    m->pos.vz += out.vz;
}

#include "tm1/PADCFG.H"

void uaPadInit(PlayerCar* car, s32 mode)
{
    car->skid[40] = 12;
    switch (mode) {
    case PAD_CONFIG_CTL_FREAK:
        car->skid[22] = 0;
        car->skid[24] = 5;
        car->skid[26] = 6;
        car->skid[28] = 8;
        car->skid[27] = 7;
        car->skid[38] = 2;
        car->skid[39] = 1;
        car->skid[29] = 12;
        car->skid[34] = 13;
        car->skid[35] = 14;
        car->skid[25] = 4;
        car->skid[33] = 11;
        car->skid[36] = 9;
        car->skid[37] = 3;
        car->skid[31] = 10;
        car->skid[32] = 10;
        break;
    case PAD_CONFIG_STANDARD:
    default:
        car->skid[22] = 1;
        car->skid[24] = 5;
        car->skid[26] = 6;
        car->skid[28] = 8;
        car->skid[27] = 7;
        car->skid[38] = 2;
        car->skid[39] = 1;
        car->skid[29] = 13;
        car->skid[25] = 14;
        car->skid[30] = 0;
        car->skid[34] = 10;
        car->skid[35] = 4;
        car->skid[36] = 3;
        car->skid[37] = 9;
        car->skid[33] = 11;
        car->skid[31] = 12;
        car->skid[32] = 12;
        break;
    case PAD_CONFIG_DEADLY_FORCE:
        car->skid[22] = 2;
        car->skid[24] = 5;
        car->skid[26] = 6;
        car->skid[28] = 8;
        car->skid[27] = 7;
        car->skid[38] = 2;
        car->skid[39] = 1;
        car->skid[29] = 12;
        car->skid[34] = 14;
        car->skid[35] = 13;
        car->skid[25] = 4;
        car->skid[33] = 11;
        car->skid[36] = 9;
        car->skid[37] = 3;
        car->skid[31] = 10;
        car->skid[32] = 10;
        break;
    case PAD_CONFIG_RUN_N_GUN:
        car->skid[22] = 3;
        car->skid[24] = 5;
        car->skid[26] = 6;
        car->skid[28] = 8;
        car->skid[27] = 7;
        car->skid[38] = 2;
        car->skid[39] = 1;
        car->skid[29] = 10;
        car->skid[25] = 9;
        car->skid[30] = 0;
        car->skid[34] = 13;
        car->skid[35] = 12;
        car->skid[36] = 11;
        car->skid[37] = 14;
        car->skid[33] = 3;
        car->skid[31] = 4;
        car->skid[32] = 4;
        break;
    }
}

void CarUpdateControlPad(PlayerCar* car)
{
    s16 idx;
    u8 turbo;
    u8 a;
    u8 b;
    u8 v;

    idx = car->playerIdx;
    turbo = 0;
    if (car->tires[0].unk3 && car->tires[1].unk3 && car->tires[2].unk3) {
        turbo = car->tires[3].unk3 != 0;
    }

    car->skid[0x0D] = GetCtlPad((s8)car->skid[0x26], idx);
    car->skid[0x0E] = GetCtlPad((s8)car->skid[0x27], idx);

    v = 0;
    if (car->skid[0x0D]) {
        v = GetCtlPad((s8)car->skid[0x18], idx) != 0;
    }
    car->skid[0x11] = v;
    v = 0;
    if (car->skid[0x0D]) {
        v = GetCtlPad((s8)car->skid[0x1A], idx) != 0;
    }
    car->skid[0x12] = v;
    v = 0;
    if (car->skid[0x0D]) {
        v = GetCtlPad((s8)car->skid[0x1C], idx) != 0;
    }
    car->skid[0x13] = v;
    v = 0;
    if (car->skid[0x0D]) {
        v = GetCtlPad((s8)car->skid[0x1B], idx) != 0;
    }
    car->skid[0x14] = v;
    v = 0;
    if (car->skid[0x0D]) {
        v = GetCtlPad((s8)car->skid[0x28], idx) != 0;
    }
    car->skid[0x17] = v;

    car->skid[0x03] = GetCtlPad((s8)car->skid[0x1C], idx);
    car->skid[0x02] = GetCtlPad((s8)car->skid[0x1B], idx);

    if (car->tires[0].unk3) {
        if (car->tires[1].unk3) {
            car->skid[0x03] = 0;
            car->skid[0x02] = 0;
        } else if (GetFrameCount() & 1) {
            car->skid[0x03] = 0;
            car->skid[0x02] = 0;
        }
    } else if (car->tires[1].unk3) {
        if (GetFrameCount() & 1) {
            car->skid[0x03] = 0;
            car->skid[0x02] = 0;
        }
    }

    a = 0;
    if (GetCtlPad((s8)car->skid[0x1F], idx)) {
        a = car->skid[0x03] != 0;
    }
    car->skid[0x06] = a;
    a = 0;
    if (GetCtlPad((s8)car->skid[0x20], idx)) {
        a = car->skid[0x02] != 0;
    }
    car->skid[0x07] = a;
    a = 0;
    if (GetCtlPad((s8)car->skid[0x21], idx)) {
        a = turbo ^ 1;
    }
    car->skid[0x08] = a;

    a = 0;
    if (GetCtlPad((s8)car->skid[0x19], idx) || GetCtlPad((s8)car->skid[0x1A], idx)) {
        if (!car->skid[0x0D] && !car->skid[0x0E]) {
            a = 1;
        }
    }
    car->skid[0x01] = a;

    b = 0;
    if ((GetCtlPad((s8)car->skid[0x1D], idx) || GetCtlPad((s8)car->skid[0x18], idx)
            || car->skid[0x01])
        && !car->skid[0x0D] && !car->skid[0x0E]) {
        b = 1;
    }
    car->skid[0x15] = b;

    v = 0;
    if (car->skid[0x15] && !car->flags[0x10] && !car->flags[0x17]) {
        v = turbo ^ 1;
    }
    car->skid[0x04] = v;

    car->skid[0x09] = GetCtlPad((s8)car->skid[0x22], idx);
    car->skid[0x0A] = GetCtlPad((s8)car->skid[0x23], idx);
    car->skid[0x0B] = GetCtlPad((s8)car->skid[0x24], idx);
    car->skid[0x0C] = GetCtlPad((s8)car->skid[0x25], idx);

    v = 0;
    if (car->skid[0x03] || car->skid[0x02]) {
        v = 1;
    }
    car->flags[0x06] = v;

    v = 0;
    if ((car->skid[0x06] || car->skid[0x07]) && car->motion.vel.vy > 0 && car->skid[0x01]) {
        v = 1;
    }
    car->skid[0x10] = v;

    if (car->skid[0x10]) {
        car->skid[0x05] = 0;
    }
    if (car->stats.unk40 <= 0) {
        car->skid[0x08] = 0;
        car->skid[0x04] = 0;
        car->skid[0x15] = 0;
        car->skid[0x09] = 0;
        car->skid[0x0A] = 0;
        car->skid[0x03] = 0;
        car->skid[0x02] = 0;
        car->skid[0x05] = 1;
    }
}

void InitBossCar(CarStats* stats)
{
    stats->unk9C = 5;
    stats->unkC4 = 3;
    stats->unk88 = 0x73;
    stats->unk8C = stats->unk88 * 2 / 3;
    stats->unk90 = 0x96;
    stats->unk98 = stats->unk88 / 3;
    stats->unkC8 = 2;
    stats->unkE0 = 0x14;
    stats->unkE4 = 5;
    stats->unk6C = 0x1B;
    stats->unk70 = 0x42;
    stats->unk74 = 0x6C;
    stats->unk7C = 0x52;
    stats->unk80 = 0x7E;
    stats->unk84 = -0x7E;
    stats->unk110 = -0x12;
    stats->unk112 = 0x69;
    stats->unk114 = 0x1B;
    stats->dropPower = 0;
    stats->unk11A = 0;
    stats->unk11C = 0x32;
    stats->unk10C = 0x320;
    stats->unk108 = 3;
    stats->unk04 = 3;
    stats->unk44 = 0xFA;
    stats->unk50 = 0x32;
    stats->unk104 = 0x46;
    stats->unk1E = 0xA;
    stats->unk5C = 0x14;
    stats->unk24 = 0x348;
}
