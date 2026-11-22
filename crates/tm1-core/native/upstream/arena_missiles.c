#include "tm1/HDP.H"
#include "tm1/arena_guns.h"
#define mlist (arena_missiles())
s32 find_free_missile(s32 i)
{
    for (; i < 10; i++) {
        if (mlist[i].life <= 0) {
            return i;
        }
    }
    return -1;
}
void turn_heatseeker(Missile* m)
{
    VECTOR3* tgt;
    s32 dx;
    s32 dy;
    s32 dz;
    s32 amin;
    s32 amax;
    s32 dist;
    s32 yaw;
    s32 pitch;
    s32 cur;

    tgt = m->target;
    dx = tgt->vx - m->cs->pos.vx;
    dy = tgt->vy - m->cs->pos.vy;
    dz = tgt->vz - m->cs->pos.vz;
    yaw = (ratan2(dx, dy) + 0x1000) % 0x1000;

    amin = abs(dx);
    amax = abs(dy);
    if (amax < amin) {
        amin = amax;
    }
    amin = amin >> 1;
    dist = abs(dx);
    if (dist >= amax) {
        dist = dist + amin;
    } else {
        dist = amax + amin;
    }
    pitch = (0x1000 - ratan2(dz, dist)) % 0x1000;
    if (pitch > 0x400 && pitch < 0xC00) {
        pitch = (0x1400 - pitch) % 0x1000;
    }

    cur = m->cs->rot.vz;
    if (cur - yaw >= 0x801) {
        yaw += 0x1000;
    }
    if (yaw - cur >= 0x801) {
        yaw -= 0x1000;
    }
    if (m->unk3C >= abs(cur - yaw)) {
        m->cs->rot.vz = yaw;
    } else if (cur < yaw) {
        m->cs->rot.vz = m->cs->rot.vz + m->unk3C;
    } else {
        m->cs->rot.vz = m->cs->rot.vz - m->unk3C;
    }
    if (m->unk2C == 2) {
        m->cs->rot.vz = m->cs->rot.vz + ((arena_rand() % (m->unk3C * 4)) - (m->unk3C * 2));
    } else if (m->unk2C == 3) {
        m->cs->rot.vz = m->cs->rot.vz + ((arena_rand() % (m->unk3C * 4)) - (m->unk3C * 2));
    }
    if (m->cs->rot.vz >= 0x1000) {
        m->cs->rot.vz = m->cs->rot.vz - 0x1000;
    }
    if (m->cs->rot.vz < 0) {
        m->cs->rot.vz = m->cs->rot.vz + 0x1000;
    }

    cur = m->cs->rot.vx;
    if (cur - pitch >= 0x801) {
        pitch += 0x1000;
    }
    if (pitch - cur >= 0x801) {
        pitch -= 0x1000;
    }
    if (m->unk3C >= abs(cur - pitch)) {
        m->cs->rot.vx = pitch;
    } else if (cur < pitch) {
        m->cs->rot.vx = m->cs->rot.vx + m->unk3C;
    } else {
        m->cs->rot.vx = m->cs->rot.vx - m->unk3C;
    }
    if (m->unk2C == 2 || m->unk2C == 3) {
        if (m->unk2C == 2) {
            m->cs->rot.vx = m->cs->rot.vx + ((arena_rand() % (m->unk3C * 2)) - m->unk3C);
        } else {
            m->cs->rot.vx = m->cs->rot.vx + ((arena_rand() % (m->unk3C * 4)) - (m->unk3C * 2));
        }
    }
    if (m->cs->rot.vx >= 0x1000) {
        m->cs->rot.vx = m->cs->rot.vx - 0x1000;
    }
    if (m->cs->rot.vx < 0) {
        m->cs->rot.vx = m->cs->rot.vx + 0x1000;
    }
    RotMatrixYXZ(&m->cs->rot, &m->cs->mat);
}
void carLockon(u8 who, VECTOR3** out)
{
    VECTOR3 d;
    VECTOR3 r;
    Cs* cs;
    VECTOR3* p;
    VECTOR3* q;
    s32 n;
    s32 i;
    s32 best;
    s32 z;
    s32 ax;
    s32 az;

    best = 0;
    if (who == 0) {
        *out = &GetPlayerCs3D(0)->pos;
        return;
    }
    p = &GetPlayerCs3D((s16)(who - 1))->pos;
    n = 1;
    *out = 0;
    for (i = 0; i < n; i++) {
        cs = GetAICs3D(i);
        q = &cs->pos;
        if (q != 0 && GetAICarInfo(i)->stats.unk40 > 0) {
            d.vx = cs->pos.vx - p->vx;
            d.vy = cs->pos.vy - p->vy;
            d.vz = cs->pos.vz - p->vz;
            mathMulVec(&GetPlayerCs3D((s16)(who - 1))->mat, &d, &r);
            z = r.vy;
            if (z >= 12) {
                ax = r.vx;
                az = ABS(z);
                if (ax < 0) {
                    ax = -ax;
                }
                if (az >= ax && az < 8001 && (best <= 0 || z < best)) {
                    best = z;
                    *out = q;
                }
            }
        }
    }
}

// Fire slot from FireMissiles, basic_missile_launch and create_FIRE_missile.
void arena_launch_missile(CarStats* stats, CarMotion* motion, CarWeap* pad, u8* flags, u8 player)
{
    int idx = find_free_missile(0);
    if (idx < 0)
        return;
    Missile* m = &mlist[idx];
    VECTOR3 local = { (s16)stats->dropPower, 0, stats->unk70 + 8 }, d;
    mathMulTransVec(&motion->mat2, &local, &d);
    Cs* cs = m->cs;
    setVector(&cs->pos, d.vx + motion->pos.vx, d.vy + motion->pos.vy, d.vz + motion->pos.vz);
    copyVector(&cs->rot, &motion->rot);
    RotMatrixYXZ(&cs->rot, &cs->mat);
    m->pos = cs->pos;
    if (pad->cur == WEAPON_FREEZE_MISSILE)
        m->target = NULL;
    else
        carLockon(player, &m->target);
    m->unk2C = pad->cur != WEAPON_FREEZE_MISSILE;
    m->life = 30;
    m->unk38 = 150;
    m->damage = pad->cur == WEAPON_HOMING_MISSILE ? 15 : (m->unk2C ? 7 : HIT_FREEZE);
    m->unk50 = player ? 1 : 50;
    m->unk3C = pad->cur == WEAPON_HOMING_MISSILE ? 50 : 10;
    m->unk48 = pad->cur;
    m->unk56 = stats->unk6C > 24 ? stats->unk6C - 24 : 0;
    pad->fireDelay = 20;
    flags[9] = 1;
    pad->ammo[pad->cur]--;
    pad->totalAmmo--;
    arena_effect(&cs->pos, 4);
}
void move_missile(Missile* m)
{
    HdCsHit* hit;

    s32 i;
    VECTOR3 fwd;
    VECTOR3 d;

    VECTOR3 pt;

    HdCsHit empty = { 0 };
    hit = &empty;

    if (m->target != 0) {
        turn_heatseeker(m);
    }
    if (m->life <= 0) {
        m->life--;
    } else {
        fwd.vz = 0;
        fwd.vx = 0;
        fwd.vy = m->unk38;
        if (m->unk38 < 150) {
            m->unk38 = m->unk38 + 5;
        } else {
            m->unk38 = 150;
        }
        mathMulTransVec(&m->cs->mat, &fwd, &d);
        setVector(&pt, m->cs->pos.vx, m->cs->pos.vy, m->cs->pos.vz);
        m->cs->pos.vx += d.vx;
        m->cs->pos.vy += d.vy;
        m->cs->pos.vz += d.vz;
        m->life = m->life - 1;
        if (m->unk56 >= 16) {
            m->cs->pos.vz -= 16;
            m->unk56 -= 16;
        } else if (m->unk56 > 0) {
            m->cs->pos.vz -= m->unk56;
            m->unk56 = 0;
        }
        for (i = 1; i >= 0; i--) {

            setVector(&pt, m->cs->pos.vx - ((d.vx * i) >> 1), m->cs->pos.vy - ((d.vy * i) >> 1),
                m->cs->pos.vz - ((d.vz * i) >> 1));
            m->unk44 = CAR_HD(&pt, m->unk50, 3);
            if (m->unk44 == m->unk50) {
                m->unk44 = 0;
            }
            if (m->unk2C == 3) {
                if (m->unk44 >= 200) {
                    m->unk44 = 0;
                } else if (m->unk44 < 0) {
                    m->unk44 = 0;
                }
            }
            if ((i & 1) && m->unk2C != 3 && m->unk44 == 0) {
                hit = HdPntTest(m->unk50, m->cs->unkC0, &pt, &m->unk44);
                if (m->unk44 == 8 && hit->tag0 == -m->unk50) {
                    m->unk44 = 0;
                }
                if (hit->tag0 == 1021) {
                    m->unk44 = 0;
                }
            }
            if (m->unk44 > 0) {
                pt.vz += 24;
                arena_effect(&pt, 2);
                m->life = -1;
                i = -1;
                bulDispatchDamage(m->unk44, hit->tag0, hit->tag1, m->damage, &d, m->unk50);
            }
        }
        if (m->unk2C != 3 && m->cs->pos.vz < 0) {
            if (shellGetCurrentLevel() != LEVEL4_CITY_PARK) {
                m->life = 0;
            } else if (m->cs->pos.vx < 6800) {
                m->life = 0;
            } else if (m->cs->pos.vx >= 8201) {
                m->life = 0;
            }
        }
        if (m->life == 0) {
            if (m->cs->pos.vz <= 0) {
                m->cs->pos.vz = 0;
            }

            arena_effect(&m->cs->pos, 2);
            m->life = -1;
        }
    }
}
void arena_move_missiles(void)
{
    for (int i = 0; i < 10; i++)
        if (mlist[i].life > 0)
            move_missile(&mlist[i]);
}
