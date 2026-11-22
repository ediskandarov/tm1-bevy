#include "tm1/arena_collision.h"

static inline s32 UpdateCollisionHeading(CarMotion* m, CarCollision* col, s32 angle)
{
    s32 difference;
    s32 result;
    BoundAngle(&angle);
    difference = abs(angle);
    result = 1;
    if (difference < 341) {
        m->rot.vz = SmoothAngleValue(m->rot.vz, (s16)col->unk8, 90);
    } else if (difference < 568) {
        m->rot.vz = SmoothAngleValue(m->rot.vz, (s16)col->unk8, 95);
    } else if (difference < 910) {
        m->rot.vz = SmoothAngleValue(m->rot.vz, (s16)col->unk8, 98);
    } else {
        m->rot.vz = SmoothAngleValue(m->rot.vz, (s16)col->unk8, 99);
    }
    return result;
}
static inline u8 UpdateCollisionAngle(CarMotion* m, CarCollision* col)
{
    u8 result;
    if (col->unk2) {
        result = 1;
        m->rot.vz = SmoothAngleValue(m->rot.vz, (s16)col->unk8, 80);
    } else if (col->unk1 && m->vel.vy) {
        result = UpdateCollisionHeading(m, col, m->rot2.vz - m->rot.vz);
    } else
        result = 1;
    return result;
}
void CheckHitDetection(Car* car, u8 isPlayer)
{
    u8* flags;
    CarStats* st;
    CarCollision* col;
    CarMotion* m;
    CarTire* t;
    Cs* cs;
    HdCsHit hit;
    PotHole* hole;
    s32 bearing;
    s32 speed;
    s32 height;
    s16 type;

    if (isPlayer) {
        flags = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
        col = &PLAYER_CAR(car)->collision;
        m = &PLAYER_CAR(car)->motion;
        t = PLAYER_CAR(car)->tires;
        cs = GetPlayerCs3D(PLAYER_CAR(car)->playerIdx);
        bearing = PLAYER_CAR(car)->bearing;
    } else {
        flags = AI_CAR(car)->flags;
        col = &AI_CAR(car)->collision;
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
        t = AI_CAR(car)->tires;
        cs = GetAICs3D(AI_CAR(car)->playerIdx);
        bearing = AI_CAR(car)->bearing;
    }
    cs->rot.vx = m->rot.vx;
    cs->rot.vy = m->rot.vy;
    cs->rot.vz = m->rot.vz;
    cs->pos.vx = m->pos.vx;
    cs->pos.vy = m->pos.vy;
    cs->pos.vz = m->pos.vz;
    cs->mat.m[0][0] = m->mat2.m[0][0];
    cs->mat.m[0][1] = m->mat2.m[0][1];
    cs->mat.m[0][2] = m->mat2.m[0][2];
    cs->mat.m[1][0] = m->mat2.m[1][0];
    cs->mat.m[1][1] = m->mat2.m[1][1];
    cs->mat.m[1][2] = m->mat2.m[1][2];
    cs->mat.m[2][0] = m->mat2.m[2][0];
    cs->mat.m[2][1] = m->mat2.m[2][1];
    cs->mat.m[2][2] = m->mat2.m[2][2];
    cs->mat.t[0] = m->mat2.t[0];
    cs->mat.t[1] = m->mat2.t[1];
    cs->mat.t[2] = m->mat2.t[2];
    hit.hit = 0;
    HdCsTest(cs, &hit, 1, &m->lastMat, &m->lastPos);
    if (hit.hit) {
        if (!isPlayer && flags[29] && st->unk22 >= 13)
            hit.hit = 0;
        type = hit.tag0;
        if (UA_IS_PICKUP(type)) {
            carGetPickup(car, isPlayer, type, hit.tag1);
            hit.hit = 0;
        } else if (type == UA_BARRICADE) {
            UAeffectBarricade(UA_BARRICADE, hit.tag1, bearing);
            col->unk12 = abs(m->vel.vy) * 3 * 33
                / (abs(m->vel.vy) < st->unkA8 ? st->unkA8 : abs(m->vel.vy));
            hit.hit = 0;
        } else if (UA_IS_PERSON(type) || type == UA_UNK_1001) {
            bulDispatchDamage(hit.obj->unkC0, type, hit.tag1, 10, &m->rot, 0);
            t[0].unk0A = MakeFakePotHole();
            t[1].unk0A = t[0].unk0A;
            t[2].unk0A = t[0].unk0A;
            t[3].unk0A = t[0].unk0A;
            t[0].unk1 = 1;
            t[1].unk1 = 1;
            t[2].unk1 = 1;
            t[3].unk1 = 1;
            hole = GetPotHoleDat(t[0].unk0A);
            if (abs(m->rot.vz) < 512) {
                hole->x = t[0].unk38 - 16;
                hole->z = t[0].unk3C - 16;
                hole->a = st->unk74 + 32;
                hole->b = m->vel.vy / 4;
            } else if (abs(m->rot.vz) >= 1537) {
                hole->x = t[1].unk38 - 16;
                hole->z = t[1].unk3C - 24;
                hole->a = st->unk74 + 32;
                hole->b = m->vel.vy / 4;
            } else {
                if (m->rot.vz > 0) {
                    hole->x = t[1].unk38 - 16;
                    hole->z = t[1].unk3C - 16;
                } else {
                    hole->x = t[0].unk38 - 24;
                    hole->z = t[0].unk3C - 16;
                }
                hole->a = m->vel.vy / 4;
                hole->b = st->unk74 + 32;
            }
            hole->c = 40;
            hit.hit = 0;
        } else if (type == UA_UNK_1400) {
            t[0].unk14 = -20;
            t[1].unk14 = -20;
            t[2].unk14 = -20;
            t[3].unk14 = -20;
            hit.hit = 0;
            m->vel.vy = 0;
        } else if (type == UA_UNK_500) {
            height = 160;
            if (!flags[10]) {
                speed = m->vel.vy / 32;
                height = st->unk5C + abs(speed);
            }
            if (flags[19] || flags[15]) {
                t[0].unk14 = -20;
                t[1].unk14 = -20;
                t[2].unk14 = -20;
                t[3].unk14 = -20;
                hit.hit = 0;
                m->vel.vy = 0;
            }
            switch (hit.histIdx) {
            case 0:
                CheckForBridge(&t[0], 1, height);
                break;
            case 1:
                CheckForBridge(&t[1], 1, height);
                break;
            case 2:
                CheckForBridge(&t[3], 1, height);
                break;
            case 3:
                CheckForBridge(&t[2], 1, height);
                break;
            default:
                CheckForBridge(&t[0], 1, height);
                CheckForBridge(&t[1], 1, height);
                break;
            }
            if (t[0].unk0 || t[1].unk0 || t[2].unk0 || t[3].unk0)
                hit.hit = 0;
        } else if (type == UA_UNK_1021) {
            if (!isPlayer && !AI_CAR(car)->unk16E && !flags[0]) {
                if (AI_CAR(car)->unk34 < UA_UNK_800 && AI_CAR(car)->uaIndex != UA_BOSS_CAR) {
                    if (col->unkE < 100 || flags[13] || flags[15])
                        hit.hit = 0;
                }
            } else
                hit.hit = 0;
        }
    }
    col->unk4 = hit.hit;
    if (hit.hit) {
        col->unk0 = 1;
        col->unk14++;
        col->unk16 = 0;
        if (hit.histIdx < cs->nhist)
            CalcHitDynamics(car, &hit, isPlayer);
        else
            col->unk0 = 0;
    } else {
        col->unk16++;
        if (col->count <= 0)
            col->unk14 = 0;
    }
    if (col->count) {
        if (isPlayer && flags[6]) {
            if (col->count < 120)
                InitNoCollision(col);
            else
                m->rotDelta.vz = 0;
        }
        if (UpdateCollisionAngle(m, col)) {

            col->count = col->count - GetFieldsLastFrame();
            if (col->count <= 0 || abs(m->vel.vy) < st->unkA4 || abs(col->unk8 - m->rot.vz) < 56)
                InitNoCollision(col);
        } else {
            InitNoCollision(col);
        }
    }
    if (col->unkE < 151)
        col->unkE = col->unkE + GetFieldsLastFrame();
}

void CalcHitDynamics(Car* car, HdCsHit* hit, u8 isPlayer)
{
    CarStats* st;
    CarMotion* m;
    CarTire* tires;
    s16 idx;
    s32 mag;
    s32 dmg;
    s32 num;
    s32 den;
    s32 scaledDamage;
    SVECTOR rot;
    VECTOR3 delta;
    VECTOR3 out;
    VECTOR3 spark;
    MATRIX mat;
    s32 ang;
    s32 rel;
    s32 bounce;
    u8 hitCs;
    u8* f;
    CarCollision* col;

    if (isPlayer) {
        f = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
        col = &PLAYER_CAR(car)->collision;
        m = &PLAYER_CAR(car)->motion;
        idx = PLAYER_CAR(car)->playerIdx;
        tires = PLAYER_CAR(car)->tires;
    } else {
        f = AI_CAR(car)->flags;
        col = &AI_CAR(car)->collision;
        st = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
        idx = AI_CAR(car)->playerIdx;
        tires = AI_CAR(car)->tires;
    }

    if (hit->unk04) {
        if (m->vel.vy >= 0) {
            ang = m->unkA8.vz;
        } else {
            ang = m->unkA8.vz + 2048;
        }
    } else {
        if (hit->normal.vy == 0) {
            hit->normal.vy = 1;
        }
        ang = ratan2(hit->normal.vx, hit->normal.vy);
    }
    BoundAngle(&ang);
    if (m->vel.vy >= 0) {
        rel = m->unkA8.vz - ang;
    } else {
        s32 reverseAngle = ang - 2048;
        rel = m->unkA8.vz - reverseAngle;
    }
    BoundAngle(&rel);
    bounce = 2048 - rel * 2;
    BoundAngle(&bounce);

    if (hit->obj != 0 && hit->obj->unkC0 < 100 && hit->obj->unkC0 != 8) {
        hitCs = CheckCSHit(car, isPlayer, hit->obj, bounce, hit->histIdx);
        col->unk3 = 1;
        m->unkA8.vz = m->rot2.vz;
        ang = m->unkA8.vz;
        BoundAngle(&ang);
    } else {
        col->unk3 = 0;
        mag = abs(m->vel.vy);
        hitCs = 0;
        if (GetFieldsLastFrame() * 570 / 100 * 32 < mag) {
            dmg = abs(rsin(bounce / 4) * abs(m->vel.vy)) * 10 / st->unkA8 / 4096;
            if (dmg > 0) {
                if (dmg - 3 > 0 && isPlayer) {
                    carTakeHit(uaGetCarMatID(idx, isPlayer), dmg - 3, &hit->normal, 1);
                }
                scaledDamage = dmg * st->unk104 / 33;
                bulDispatchDamage(hit->obj->unkC0, hit->tag0, hit->tag1, scaledDamage, &m->rot, 0);
                if (f[10]) {
                    tires[0].unk3 = 1;
                    tires[1].unk3 = 1;
                    tires[2].unk3 = 1;
                    tires[3].unk3 = 1;
                    if (tires[0].unk26 < 2) {
                        tires[0].unk14 = 0;
                    }
                    if (tires[1].unk26 < 2) {
                        tires[1].unk14 = 0;
                    }
                    if (tires[2].unk26 < 2) {
                        tires[2].unk14 = 0;
                    }
                    if (tires[3].unk26 < 2) {
                        tires[3].unk14 = 0;
                    }
                    tires[0].unk14 = tires[0].unk14 / 2;
                    tires[1].unk14 = tires[1].unk14 / 2;
                    tires[2].unk14 = tires[2].unk14 / 2;
                    tires[3].unk14 = tires[3].unk14 / 2;
                }
            }
        }
        if (!isPlayer) {
            CheckIfLostAICar(car);
            if (bounce >= 1707) {
                bounce = 1706;
            } else if (bounce < -1706) {
                bounce = -1706;
            }
        }
        RicochetOffObject(car, isPlayer, 0, bounce, hit->histIdx);
    }

    if (hitCs == 0) {
        rot.vx = 0;
        rot.vy = 0;
        rot.vz = ang;
        RotMatrixYXZ(&rot, &mat);
        delta.vx = 0;
        if (hit->unk04 && col->unk3 == 0) {
            hit->depth += abs(m->vel.vy / 32);
        }
        delta.vy = -(hit->depth + 8);
        delta.vz = 0;
        mathMulTransVec(&mat, &delta, &out);
        m->pos.vx += out.vx;
        m->pos.vy += out.vy;
        m->pos.vz += out.vz;
        delta.vx = 0;
        delta.vy = 0;
        delta.vz = 0;
        switch (hit->histIdx) {
        case 0:
            delta.vx = 8;
            spark.vx = tires[0].unk38;
            spark.vy = tires[0].unk3C;
            spark.vz = tires[0].unk40 + st->unk6C;
            break;
        case 1:
            delta.vx = -8;
            spark.vx = tires[1].unk38;
            spark.vy = tires[1].unk3C;
            spark.vz = tires[1].unk40 + st->unk6C;
            break;
        case 2:
            delta.vx = -8;
            if (hit->unk04) {
                delta.vy = (abs(hit->depth) + 8) * 2;
            }
            spark.vx = tires[3].unk38;
            spark.vy = tires[3].unk3C;
            spark.vz = tires[3].unk40 + st->unk6C;
            break;
        case 3:
            delta.vx = 8;
            if (hit->unk04) {
                delta.vy = (abs(hit->depth) + 8) * 2;
            }
            spark.vx = tires[2].unk38;
            spark.vy = tires[2].unk3C;
            spark.vz = tires[2].unk40 + st->unk6C;
            break;
        case 4:
            delta.vy = -16;
            spark.vx = (tires[0].unk38 + tires[1].unk38) / 2;
            spark.vy = (tires[0].unk3C + tires[1].unk3C) / 2;
            spark.vz = (tires[0].unk40 + tires[1].unk40) / 2 + st->unk6C;
            break;
        case 5:
            delta.vy = 8;
            spark.vx = (tires[2].unk38 + tires[3].unk38) / 2;
            spark.vy = (tires[2].unk3C + tires[3].unk3C) / 2;
            spark.vz = (tires[2].unk40 + tires[3].unk40) / 2 + st->unk6C;
            break;
        case 6:
            delta.vx = 8;
            spark.vx = (tires[0].unk38 + tires[2].unk38) / 2;
            spark.vy = (tires[0].unk3C + tires[2].unk3C) / 2;
            spark.vz = (tires[0].unk40 + tires[2].unk40) / 2 + st->unk6C;
            break;
        case 7:
            delta.vx = -8;
            spark.vx = (tires[1].unk38 + tires[3].unk38) / 2;
            spark.vy = (tires[1].unk3C + tires[3].unk3C) / 2;
            spark.vz = (tires[1].unk40 + tires[3].unk40) / 2 + st->unk6C;
            break;
        default:
            spark.vx = 0;
            spark.vy = 0;
            spark.vz = tires[0].unk40 + st->unk6C;
            break;
        }
        num = abs(m->vel.vy) * 3 * 33;
        if (abs(m->vel.vy) < st->unkA8) {
            den = st->unkA8;
        } else {
            den = abs(m->vel.vy);
        }
        col->unk12 = num / den;
        mathMulTransVec(&m->mat2, &delta, &out);
        m->pos.vx += out.vx;
        m->pos.vy += out.vy;
        m->pos.vz += out.vz;
        if (GetFieldsLastFrame() * 380 / 100 * 32 < m->vel.vy) {
            do_simple_spark(&spark);
            if (hit->unk04) {
                do_puff(&spark);
            }
        }
    }
}

void CheckIfLostAICar(Car* car)
{
    AI_CAR(car)->unk180++;
    if (AI_CAR(car)->unk180 >= 4) {
        StartLostCheck(car);
    }
    if (AI_CAR(car)->unk4D) {
        AI_CAR(car)->unk182 = 80;
    }
}

u8 CheckCSHit(Car* car, u8 isPlayer, Cs* obj, s32 a, s32 b)
{
    u8 otherIsPlayer;
    s16 idx;
    Car* info;
    u8 ret;
    u8 hit;
    s32 bb;

    hit = uaIsCarCS(obj, &otherIsPlayer, &idx);
    bb = b;
    if (hit) {
        if (otherIsPlayer) {
            info = GetPlayerInfo(idx);
        } else {
            info = GetAICarInfo(idx);
        }
        DoCsHitCalculations(car, isPlayer, info, otherIsPlayer, a, bb);
        ret = 0;
    } else {
        RicochetOffObject(car, isPlayer, 0, a, b);
        ret = 0;
    }
    return ret;
}

void DoCsHitCalculations(
    Car* car, u8 isPlayer, Car* other, u8 otherIsPlayer, s32 collisionAngle, s16 mode)
{
    u8* f;
    CarStats* st;
    CarMotion* m;
    CarStats* ost;
    CarMotion* om;
    u8 front;
    u8 driftMe;
    u8 driftOther;
    u16 idx;
    s16 oidx;
    CarCollision* col;
    CarCollision* ocol;
    u8* of;
    s32 dead;
    u8 back;
    s32 otherBounceAngle;
    s32 bounceAngle;
    s32 impulse;
    s32 otherImpulse;
    s32 angleDifference;
    s32 driftAngle;
    VECTOR3 spark;
    s32 relativeAngle;

    driftMe = 0;
    driftOther = 0;
    if (isPlayer) {
        col = &PLAYER_CAR(car)->collision;
        f = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
        idx = PLAYER_CAR(car)->playerIdx;
        dead = PLAYER_CAR(car)->uaIndex;
    } else {
        f = AI_CAR(car)->flags;
        st = &AI_CAR(car)->stats;
        col = &AI_CAR(car)->collision;
        m = &AI_CAR(car)->motion;
        idx = AI_CAR(car)->playerIdx;
        dead = AI_CAR(car)->uaIndex;
    }
    if (otherIsPlayer) {
        of = PLAYER_CAR(other)->flags;
        ost = &PLAYER_CAR(other)->stats;
        ocol = &PLAYER_CAR(other)->collision;
        om = &PLAYER_CAR(other)->motion;
        oidx = PLAYER_CAR(other)->playerIdx;
    } else {
        of = AI_CAR(other)->flags;
        ost = &AI_CAR(other)->stats;
        ocol = &AI_CAR(other)->collision;
        om = &AI_CAR(other)->motion;
        oidx = AI_CAR(other)->playerIdx;
    }
    if (isPlayer) {
        if (!otherIsPlayer && !of[0]) {
            of[24] = 1;
        }
    } else if (otherIsPlayer && !f[0]) {
        f[24] = 1;
    }
    if ((u16)mode < 2 || mode == 4)
        front = 1;
    else
        front = 0;
    back = mode == 2 || mode == 3;
    if (st->monster) {
        if (f[27]) {
            return;
        }
        if ((front && m->vel.vy > 0) || (back && m->vel.vy < 0)) {
            InitMonsterSmash(car, isPlayer, other, otherIsPlayer, mode);
            return;
        }
    } else if (of[28]) {
        return;
    }

    if (isPlayer || otherIsPlayer) {
        s32 damage;
        if (abs(m->vel.vy) > GetFieldsLastFrame() * 570 / 100 * 32
            || abs(om->vel.vy) > GetFieldsLastFrame() * 570 / 100 * 32) {
            if (st->unk108 > ost->unk108) {
                if (!f[27]) {
                    carTakeHit(uaGetCarMatID(idx, isPlayer), 1, &m->unkA8, 1);
                }
                if (abs(m->vel.vy) > (GetFieldsLastFrame() * 570 / 100 * 32) && !of[27]) {
                    damage = 12 * m->vel.vy / st->unkA8;
                    if (damage > 0) {
                        carTakeHit(uaGetCarMatID(oidx, otherIsPlayer), damage, &m->unkA8, 1);
                    }
                }
            } else if (st->unk108 < ost->unk108) {
                if (abs(om->vel.vy) > (GetFieldsLastFrame() * 570 / 100 * 32) && !f[27]) {
                    damage = 12 * om->vel.vy / ost->unkA8;
                    if (damage > 0) {
                        carTakeHit(uaGetCarMatID(idx, isPlayer), 11, &om->unkA8, 1);
                    }
                }
                if (!of[27]) {
                    carTakeHit(uaGetCarMatID(oidx, otherIsPlayer), 1, &om->unkA8, 1);
                }
            } else {
                if (abs(om->vel.vy) > (GetFieldsLastFrame() * 570 / 100 * 32) && !f[27]) {
                    damage = 6 * om->vel.vy / ost->unkA8;
                    if (damage > 0) {
                        carTakeHit(uaGetCarMatID(idx, isPlayer), damage, &om->unkA8, 1);
                    }
                }
                if (abs(m->vel.vy) > (GetFieldsLastFrame() * 570 / 100 * 32) && !of[27]) {
                    damage = 6 * m->vel.vy / st->unkA8;
                    if (damage > 0) {
                        carTakeHit(uaGetCarMatID(oidx, otherIsPlayer), damage, &m->unkA8, 1);
                    }
                }
            }
        }
    }

    {
        s32 heading;
        s32 otherHeading;
        if (st->unkA4 < abs(m->vel.vy))
            heading = m->unkA8.vz;
        else
            heading = m->rot.vz;
        if (ost->unkA4 < abs(om->vel.vy))
            otherHeading = om->unkA8.vz;
        else
            otherHeading = om->rot.vz;
        relativeAngle = heading - otherHeading;
    }
    BoundAngle(&relativeAngle);

    if (m->vel.vy != 0 && (om->vel.vy == 0 || of[0])) {
        of[1] = 1;
        SetFullCarDrift(om);
        om->rot2.vz = m->rot2.vz;
        bounceAngle = 0;
        otherBounceAngle = 0;
        if (st->unk108 > ost->unk108) {
            if (st->unk108 - 1 > ost->unk108) {
                om->vel.vy = 3 * m->vel.vy;
                m->vel.vy = 4 * m->vel.vy / 5;
            } else {
                om->vel.vy = 2 * m->vel.vy;
                m->vel.vy /= 2;
            }
        } else if (ost->unk108 > st->unk108) {
            om->vel.vy = m->vel.vy / 4;
            m->vel.vy /= 8;
        } else {
            om->vel.vy = m->vel.vy;
            m->vel.vy /= 2;
        }
    } else if (om->vel.vy != 0 && (m->vel.vy == 0 || f[0])) {
        f[1] = 1;
        SetFullCarDrift(m);
        m->rot2.vz = om->rot2.vz;
        bounceAngle = 0;
        otherBounceAngle = 0;
        if (ost->unk108 > st->unk108) {
            if (ost->unk108 - 1 > st->unk108) {
                m->vel.vy = 3 * om->vel.vy;
                om->vel.vy = 4 * om->vel.vy / 5;
            } else {
                m->vel.vy = 2 * om->vel.vy;
                om->vel.vy /= 2;
            }
        } else if (st->unk108 < ost->unk108) {
            m->vel.vy = om->vel.vy / 4;
            om->vel.vy /= 8;
        } else {
            m->vel.vy = om->vel.vy;
            om->vel.vy /= 2;
        }
    } else {
        s32 angleMagnitude = abs(collisionAngle);
        if (angleMagnitude < 1024) {
            otherBounceAngle = collisionAngle;
            bounceAngle = otherBounceAngle;
            if (st->unk108 > ost->unk108) {
                bounceAngle = otherBounceAngle / 2;
                om->vel.vy /= 2;
            } else if (st->unk108 < ost->unk108) {
                otherBounceAngle /= 2;
                m->vel.vy /= 2;
            }
            angleDifference = abs(relativeAngle);
            if (angleDifference < 1024) {
                otherBounceAngle = -otherBounceAngle;
            }
            {
                s32 speedA;
                s32 speedB;
                speedA = m->vel.vy;
                speedB = om->vel.vy;
                if (speedA < 0)
                    speedA = -speedA;
                if (speedB < 0)
                    speedB = -speedB;
                if (2 * speedB < speedA) {
                    om->rot2.vz = m->unkA8.vz - bounceAngle;
                    if (m->vel.vy < 0) {
                        om->rot2.vz -= 2048;
                        if (angleDifference < 1024) {
                            om->rot2.vz -= 2 * bounceAngle;
                        }
                    } else {
                        if (angleDifference >= 1025) {
                            om->rot2.vz -= 2 * bounceAngle;
                        }
                    }
                    otherBounceAngle = 0;
                } else {
                    speedA = om->vel.vy;
                    speedB = m->vel.vy;
                    if (speedA < 0)
                        speedA = -speedA;
                    if (speedB < 0)
                        speedB = -speedB;
                    if (2 * speedB < speedA) {
                        m->rot2.vz = om->unkA8.vz - otherBounceAngle;
                        if (om->vel.vy < 0) {
                            m->rot2.vz -= 2048;
                            if (angleDifference < 1024) {
                                m->rot2.vz -= 2 * otherBounceAngle;
                            }
                        } else {
                            if (angleDifference >= 1025) {
                                m->rot2.vz -= 2 * otherBounceAngle;
                            }
                        }
                        bounceAngle = 0;
                    }
                }
                speedA = m->vel.vy;
                speedB = om->vel.vy;
                if (speedA < 0)
                    speedA = -speedA;
                if (speedB < 0)
                    speedB = -speedB;
                if (speedB < speedA) {
                    s32 direction = relativeAngle;
                    if (direction < 0)
                        direction = -direction;
                    if (direction < 1024) {
                        om->vel.vy = m->vel.vy;
                    } else {
                        om->vel.vy = -m->vel.vy;
                    }
                } else {
                    s32 direction = relativeAngle;
                    if (direction < 0)
                        direction = -direction;
                    if (direction < 1024) {
                        m->vel.vy = om->vel.vy;
                    } else {
                        m->vel.vy = -om->vel.vy;
                    }
                }
            }
        } else {
            s32 separation = abs(relativeAngle);
            if (separation < 682) {
                impulse = m->vel.vy - om->vel.vy;
                otherImpulse = om->vel.vy - m->vel.vy;
                if (st->unk108 > ost->unk108) {
                    impulse /= 2;
                    otherImpulse *= 2;
                } else if (st->unk108 < ost->unk108) {
                    impulse *= 2;
                    otherImpulse /= 2;
                }
                m->rot2.vz = m->rot.vz;
                m->vel.vy -= impulse;
                om->rot2.vz = om->rot.vz;
                om->vel.vy -= otherImpulse;
                if (mode == 1 || mode == 3) {
                    bounceAngle = -170;
                    otherBounceAngle = 170;
                } else {
                    bounceAngle = 170;
                    otherBounceAngle = -170;
                }
            } else if (separation < 1365) {
                bounceAngle = 0;
                otherBounceAngle = 0;
                if (front) {
                    om->vel.vy = m->vel.vy;
                    om->rot2.vz = m->rot2.vz;
                    of[1] = 1;
                    SetFullCarDrift(om);
                    if (st->unk108 > ost->unk108) {
                        if (st->unk108 - 1 > ost->unk108) {
                            m->vel.vy = 4 * m->vel.vy / 5;
                        } else {
                            m->vel.vy = m->vel.vy / 2;
                        }
                    } else {
                        m->vel.vy = m->vel.vy / 4;
                    }
                    if (st->unk104 >= ost->unk104 && m->vel.vy > st->unkC0) {
                        driftOther = 1;
                    }
                } else {
                    m->vel.vy = om->vel.vy;
                    m->rot2.vz = om->rot2.vz;
                    f[1] = 1;
                    SetFullCarDrift(m);
                    if (ost->unk108 > st->unk108) {
                        if (ost->unk108 - 1 > st->unk108) {
                            om->vel.vy = 4 * om->vel.vy / 5;
                        } else {
                            om->vel.vy = om->vel.vy / 2;
                        }
                    } else {
                        om->vel.vy = om->vel.vy / 4;
                    }
                    if (ost->unk104 >= st->unk104 && om->vel.vy > ost->unkC0) {
                        driftMe = 1;
                    }
                }
                if (st->unk108 > ost->unk108) {
                    if (st->unk108 - 1 > ost->unk108) {
                        om->vel.vy *= 3;
                    } else {
                        om->vel.vy *= 2;
                    }
                } else if (ost->unk108 > st->unk108) {
                    if (ost->unk108 - 1 < st->unk108) {
                        m->vel.vy = 3 * m->vel.vy;
                    } else {
                        m->vel.vy = 2 * m->vel.vy;
                    }
                }
                if (mode == 0) {
                    bounceAngle += 170;
                } else {
                    bounceAngle -= 170;
                }
            } else {
                otherImpulse = m->vel.vy + om->vel.vy;
                impulse = otherImpulse;
                if (st->unk108 > ost->unk108) {
                    if (st->unk108 - 1 > ost->unk108) {
                        impulse = m->vel.vy / 8;
                        otherImpulse *= 3;
                    } else {
                        impulse = m->vel.vy / 4;
                        otherImpulse *= 2;
                    }
                    of[1] = 1;
                    SetFullCarDrift(om);
                    if (m->vel.vy > st->unkC0) {
                        driftOther = 1;
                    }
                } else if (ost->unk108 > st->unk108) {
                    if (ost->unk108 - 1 > st->unk108) {
                        otherImpulse = om->vel.vy / 8;
                        impulse *= 3;
                    } else {
                        otherImpulse = om->vel.vy / 4;
                        impulse *= 2;
                    }
                    f[1] = 1;
                    SetFullCarDrift(m);
                    if (om->vel.vy > ost->unkC0) {
                        driftMe = 1;
                    }
                }
                if (m->vel.vy - impulse > om->vel.vy - otherImpulse) {
                    om->vel.vy = m->vel.vy;
                    om->rot2.vz = om->rot.vz - 2048;
                    otherBounceAngle = 0;
                    bounceAngle = 0;
                    m->vel.vy -= impulse;
                } else {
                    m->vel.vy = om->vel.vy;
                    m->rot2.vz = m->rot.vz - 2048;
                    bounceAngle = 0;
                    otherBounceAngle = 0;
                    om->vel.vy -= otherImpulse;
                }

                if (mode == 1) {
                    bounceAngle += 170;
                    otherBounceAngle -= 170;
                } else {
                    bounceAngle -= 170;
                    otherBounceAngle += 170;
                }
            }
        }
    }

    if (driftMe) {
        f[23] = 1;
        m->unk02 = m->rot2.vz;
        relativeAngle = m->rot.vz - m->rot2.vz;
        BoundAngle(&relativeAngle);
        if (relativeAngle > 0) {
            m->rotDelta.vy = 1;
            driftAngle = m->rot2.vz + 1024;
        } else {
            m->rotDelta.vy = -1;
            driftAngle = m->rot2.vz - 1024;
        }
        m->unk04 = driftAngle;
    }
    if (driftOther) {
        of[23] = 1;
        om->unk02 = om->rot2.vz;
        relativeAngle = om->rot.vz - om->rot2.vz;
        BoundAngle(&relativeAngle);
        if (relativeAngle > 0) {
            om->rotDelta.vy = 1;
            driftAngle = om->rot2.vz + 1024;
        } else {
            om->rotDelta.vy = -1;
            driftAngle = om->rot2.vz - 1024;
        }
        om->unk04 = driftAngle;
    }
    if (col->unkE < 150 && col->unk10 == oidx) {
        spark.vx = (m->pos.vx + om->pos.vx) / 2;
        spark.vy = (m->pos.vy + om->pos.vy) / 2;
        spark.vz = (m->pos.vz + om->pos.vz) / 2;
        do_simple_spark(&spark);
        m->rot2.vz = m->rot.vz;
        switch (mode) {
        case 0:
            bounceAngle = (m->vel.vy == 0) ? 0x238 : 0;
            m->rot2.vz += 512;
            break;
        case 1:
            bounceAngle = -0x238;
            if (m->vel.vy != 0) {
                bounceAngle = 0;
            }
            m->rot2.vz -= 512;
            break;
        case 2:
            bounceAngle = -512;
            m->rot2.vz -= 512;
            break;
        case 3:
            bounceAngle = 512;
            m->rot2.vz += 512;
            break;
        case 4:
            bounceAngle = (m->vel.vy == 0) ? 0x155 : 0;
            m->rot2.vz += 1024;
            break;
        case 5:
            bounceAngle = 0;
            m->rot2.vz += 512;
            break;
        case 6:
            bounceAngle = 1024;
            m->rot2.vz += 512;
            break;
        case 7:
            bounceAngle = -1024;
            m->rot2.vz -= 512;
            break;
        default:
            bounceAngle = 1024;
            m->rot2.vz += 1024;
            break;
        }

        if (m->vel.vy > 0) {
            m->vel.vy = abs(m->vel.vy) < GetFieldsLastFrame() * 1140 / 100 * 32
                ? GetFieldsLastFrame() * 1140 / 100 * 32
                : abs(m->vel.vy);
        } else {
            m->vel.vy = abs(m->vel.vy) < -(GetFieldsLastFrame() * 1140 / 100 * 32)
                ? abs(m->vel.vy)
                : -(GetFieldsLastFrame() * 1140 / 100 * 32);
        }
    }

    col->unkE = 0;
    col->unk10 = oidx;
    ocol->unkE = 0;
    ocol->unk10 = idx;
    if (m->vel.vy > st->unkA8) {
        m->vel.vy = st->unkA8;
    }
    if (om->vel.vy > ost->unkA8) {
        om->vel.vy = ost->unkA8;
    }
    RicochetOffObject(car, isPlayer, 1, bounceAngle, mode);
    ocol->unk0 = 1;
    RicochetOffObject(other, otherIsPlayer, 1, otherBounceAngle, mode);
    RotateCarsAwayFromCollision(m, om);
}

void RotateCarsAwayFromCollision(CarMotion* a, CarMotion* b)
{
    s32 ang;
    s32 dx;
    s32 dy;
    s32 ax;
    s32 bx;
    s32 ay;
    s32 by;

    do {
        ax = a->pos.vx;
        bx = b->pos.vx;
        ay = a->pos.vy;
        by = b->pos.vy;
    } while (0);
    dy = ay - by;
    dx = ax - bx;
    if (dy == 0) {
        dy = 1;
    }
    ang = ratan2(dx, dy) - a->rot.vz;
    BoundAngle(&ang);
}

void RicochetOffObject(Car* car, u8 isPlayer, u8 doHit, s32 delta, s16 mode)
{
    CarMotion* m;
    u8* f;
    CarCollision* c;
    u8* skid;
    CarStats* st;
    s32 ang;
    s32 sub;
    u8 ok;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        f = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
        c = &PLAYER_CAR(car)->collision;
        skid = PLAYER_CAR(car)->skid;
    } else {
        m = &AI_CAR(car)->motion;
        f = AI_CAR(car)->flags;
        c = &AI_CAR(car)->collision;
        st = &AI_CAR(car)->stats;
        skid = AI_CAR(car)->skid;
        AI_CAR(car)->unk16D = 1;
    }

    m->rot2.vz += delta;
    BoundAngle(&m->rot2.vz);
    f[1] = 1;
    SetCollisionBounce(car, isPlayer, delta);

    if (isPlayer && f[6] && m->vel.vy < GetFieldsLastFrame() * 570 / 100 * 32
        && ((u16)mode < 2 || mode == 4)) {
        c->unk2 = 1;
    } else {
        c->unk2 = 0;
    }

    if ((u16)mode < 2 || mode == 4 || skid[1]) {
        if (c->unk2 || abs(delta) < 682) {
            m->rot2.vz -= delta / 3;
            c->unk8 = m->rot2.vz;
        } else if (abs(delta) < 1365) {
            c->unk8 = m->rot2.vz - delta / 2;
        } else {
            c->unk8 = m->rot2.vz - 2048;
        }
        BoundSAngle(&c->unk8);
        c->unk1 = 1;
    }

    ang = m->rot.vz - m->rot2.vz;
    BoundAngle(&ang);

    if (c->unk2) {
        if (f[6]) {
            if ((ang < 0 && skid[2]) || (ang > 0 && skid[3])) {
                c->unk8 = m->rot2.vz - 2048;
                BoundSAngle(&c->unk8);
            }
        }
    }

    if (isPlayer && f[6]) {
        if (skid[2]) {
            c->unk8 = m->rot.vz - 512;
        } else {
            c->unk8 = m->rot.vz + 512;
        }
        BoundSAngle(&c->unk8);
        c->unk1 = 1;
    }

    if (doHit && m->vel.vy > st->unkC0) {
        if (ang >= 1764) {
            ok = 0;
        } else if (ang >= 683) {
            ok = 1;
        } else if (ang < -1763) {
            ok = 0;
        } else {
            ok = ang < -682;
        }
        if (ok) {
            c->unkC = 1;
        } else {
            c->unkC = 0;
        }
    } else {
        c->unkC = 0;
    }

    sub = 0;
    if (m->vel.vy < 0) {
        sub = m->vel.vy * 3 / 4;
    } else if (m->vel.vy >= GetFieldsLastFrame() * 570 / 100 * 32) {
        if (abs(delta) < 512) {
            sub = m->vel.vy / 4;
        } else if (abs(delta) < 1024) {
            sub = m->vel.vy / 3;
        } else {
            sub = m->vel.vy / 2;
        }
    }
    m->vel.vy -= sub;
    SetCollisCount(c);
}

void SetCollisionBounce(Car* car, u8 isPlayer, s32 mag)
{
    mag = abs(mag);
    if (mag < 227) {
        SetBounce(car, 1, 2, 2, isPlayer);
    } else if (mag < 512) {
        SetBounce(car, 2, 2, 2, isPlayer);
    } else if (mag < 1024) {
        SetBounce(car, 3, 2, 2, isPlayer);
    } else if (mag < 1536) {
        SetBounce(car, 3, 2, 2, isPlayer);
    } else {
        SetBounce(car, 5, 2, 2, isPlayer);
    }
}

void SetCollisCount(CarCollision* c)
{
    if (c->count <= 0) {
        c->count = 150;
    }
}
