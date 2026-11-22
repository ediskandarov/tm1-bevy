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
static void car_lockon(u8 who, int rear, VECTOR3** out)
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
    n = GetNumAICars();
    *out = 0;
    // Retail checks the other human when no AI car was in the aiming cone.
    for (i = 0; i < n + GetNumPlayers(); i++) {
        if (i >= n) {
            if (best > 0)
                break;
            if (i - n == who - 1)
                continue;
            cs = GetPlayerCs3D(i - n);
        } else {
            if (GetAICarInfo(i)->stats.unk40 <= 0)
                continue;
            cs = GetAICs3D(i);
        }
        q = &cs->pos;
        d.vx = cs->pos.vx - p->vx;
        d.vy = cs->pos.vy - p->vy;
        d.vz = cs->pos.vz - p->vz;
        mathMulVec(&GetPlayerCs3D((s16)(who - 1))->mat, &d, &r);
        z = rear ? -r.vy : r.vy;
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

// WEAPON.c startContrail and drop_contrail connect seven triangular rings
// into six textured sections. World-relative offsets retain PS1 short width.
static void update_trail(Missile* m)
{
    if (--m->trail.delay == 0) {
        int head = (m->trail.head + 1) % 6;
        m->trail.head = head;
        m->trail.ends[head][0] = m->trail.ring_index;
        m->trail.ring_index = (m->trail.ring_index + 1) % 7;
        m->trail.ends[head][1] = m->trail.ring_index;
        m->trail.delay = 2;
    }
    VECTOR3 local[] = { { 0, 0, 8 }, { 8, 0, -8 }, { -8, 0, -8 } };
    for (int i = 0; i < 3; i++) {
        VECTOR3 d;
        mathMulTransVec(&m->cs->mat, &local[i], &d);
        SVECTOR* v = &m->trail.ring[m->trail.ring_index][i];
        setVector(v, m->cs->pos.vx + d.vx - m->pos.vx, m->cs->pos.vy + d.vy - m->pos.vy,
            m->cs->pos.vz + d.vz - m->pos.vz);
    }
}

static void start_trail(Missile* m, int color)
{
    VECTOR3 back = { 0, -200, 0 }, d;
    mathMulTransVec(&m->cs->mat, &back, &d);
    for (int i = 0; i < 7; i++)
        for (int j = 0; j < 3; j++)
            setVector(&m->trail.ring[i][j], d.vx, d.vy, d.vz);
    for (int j = 0; j < 6; j++)
        m->trail.ends[j][0] = m->trail.ends[j][1] = 0;
    m->trail.head = 0;
    m->trail.ring_index = 1;
    m->trail.delay = 1;
    m->trail.color = color;
    m->trail.visible = m->model != 312;
    update_trail(m);
}

// WEAPON.c create_LOS_missile used by TARGETS.c static_cop_fire.
void arena_cop_missile(int owner, int yaw, const VECTOR3* pos)
{
    int index = find_free_missile(0);
    if (index < 0)
        return;
    Missile* m = &mlist[index];
    m->model = 201;
    m->cs->pos = *pos;
    setVector(&m->cs->rot, 0, 0, yaw);
    RotMatrixYXZ(&m->cs->rot, &m->cs->mat);
    m->pos = *pos;
    m->unk2C = m->unk48 = m->unk56 = 0;
    m->life = 30;
    m->unk38 = 150;
    m->damage = 2;
    m->target = NULL;
    m->unk50 = owner;
    start_trail(m, 0x0a320a);
}

// Common position and orientation writes in INTERACT.c and basic_missile_launch.
static void mount_missile(Missile* m, CarMotion* motion, VECTOR3 local)
{
    VECTOR3 d;
    mathMulTransVec(&motion->mat2, &local, &d);
    Cs* cs = m->cs;
    setVector(&cs->pos, d.vx + motion->pos.vx, d.vy + motion->pos.vy, d.vz + motion->pos.vz);
    copyVector(&cs->rot, &motion->rot);
    RotMatrixYXZ(&cs->rot, &cs->mat);
    m->pos = cs->pos;
    m->unk56 = 0;
    m->model = 201;
}

void arena_special_missiles(int kind, int owner, CarStats* stats, CarMotion* motion, CarWeap* pad, u8* flags)
{
    int cost = arena_player_special_cost(kind);
    if (pad->ammo[WEAPON_SPECIAL] >= cost) {
        VECTOR3* target;
        car_lockon(owner < 50, 0, &target);
        VECTOR3 mounts[] = { { (s16)stats->dropPower, 0, stats->unk70 + 8 },
            { stats->unk110, stats->unk112, stats->unk114 },
            { -stats->unk110, stats->unk112, stats->unk114 } };
        int count = kind == UA_HUMVEE ? 3 : kind == UA_BOSS_CAR ? 2 : 1;
        for (int i = 0; i < count; i++) {
            int index = find_free_missile(0);
            if (index < 0)
                break;
            Missile* m = &mlist[index];
            mount_missile(m, motion, mounts[kind == UA_BOSS_CAR ? 0 : i]);
            if (kind == UA_BOSS_CAR && i == 1) {
                car_lockon(owner < 50, 1, &target);
                m->cs->rot.vz += 2048;
                RotMatrixYXZ(&m->cs->rot, &m->cs->mat);
            }
            m->target = target;
            m->unk50 = owner;
            m->life = 30;
            m->unk38 = 150;
            switch (kind) {
            case UA_BOSS_CAR:
            case UA_HUMVEE:
                m->unk2C = 2;
                m->unk3C = 35;
                m->damage = kind == UA_BOSS_CAR ? 10 : 3;
                break;
            case UA_CORVETTE:
                m->unk2C = 3;
                m->unk3C = 100;
                m->unk38 = 75;
                m->life = 80;
                m->damage = 10;
                m->unk48 = WEAPON_SINGING_MISSILE;
                arena_record_effect(&m->pos, 12 + WEAPON_SINGING_MISSILE);
                break;
            case UA_MAD_MAX:
                m->unk2C = 1;
                m->unk3C = 10;
                m->damage = 16;
                m->unk48 = WEAPON_SPECIAL;
                m->model = 312;
                arena_record_effect(&m->pos, 12 + WEAPON_HOMING_MISSILE);
                do_smoke(&m->pos);
                break;
            }
            start_trail(m, kind == UA_CORVETTE ? 0xcdcdff : 0x808080);
            if (i == 0 && kind != UA_BOSS_CAR) {
                pad->ammo[WEAPON_SPECIAL] -= cost;
                arena_special_effect(&m->pos, owner);
            }
        }
    }
    pad->fireDelay = 20;
    flags[9] = 1;
}

// Fire slot from FireMissiles, basic_missile_launch and create_FIRE_missile.
void arena_launch_missile(CarStats* stats, CarMotion* motion, CarWeap* pad, u8* flags, u8 owner)
{
    int player = owner < 50;
    int idx = find_free_missile(0);
    if (idx < 0)
        return;
    Missile* m = &mlist[idx];
    VECTOR3 local = { (s16)stats->dropPower, 0, stats->unk70 + 8 };
    mount_missile(m, motion, local);
    Cs* cs = m->cs;
    const int slot = pad->cur;
    static const int damage[] = { 7, HIT_FREEZE, 15, 25, 3, 7 };
    static const int trail_color[] = { 0x808080, 0x20209f, 0x400040, 0xfa0000, 0x108010, 0x808080 };
    const int rear = slot == WEAPON_REAR_MISSILE;
    if (rear) {
        cs->rot.vz = (cs->rot.vz + 2048) & 4095;
        RotMatrixYXZ(&cs->rot, &cs->mat);
    }
    m->unk2C = slot == WEAPON_FIRE_MISSILE || slot == WEAPON_HOMING_MISSILE || rear;
    if (m->unk2C)
        car_lockon(player, rear, &m->target);
    else
        m->target = NULL;
    m->life = 30;
    m->unk38 = 150;
    m->damage = damage[slot];
    m->unk50 = owner;
    m->unk3C = slot == WEAPON_HOMING_MISSILE ? 50 : 10;
    m->unk48 = slot;
    m->unk56 = stats->unk6C > 24 ? stats->unk6C - 24 : 0;
    pad->fireDelay = 20;
    flags[9] = 1;
    pad->ammo[pad->cur]--;
    pad->totalAmmo--;
    start_trail(m, trail_color[slot]);
    arena_record_effect(&cs->pos, 12 + slot);
}
void move_missile(Missile* m)
{
    HdCsHit* hit;

    s32 i;
    VECTOR3 fwd;
    VECTOR3 d;

    VECTOR3 pt;

    int exploded = 0;
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
                arena_effect(&pt, m->damage < 0 ? 55 : m->unk44 == 8 ? 56 : 28);
                if (m->unk44 != 8) {
                    if (m->unk48 == WEAPON_SPECIAL)
                        arena_effect(&pt, 31);
                    else
                        do_smoke(&pt);
                }
                exploded = 1;
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

            exploded = 0;
            arena_effect(&m->cs->pos, 56);
            m->life = -1;
        }
    }
    if (m->trail.visible)
        update_trail(m);
    if (exploded)
        arena_weapon_impact_sound(&m->cs->pos, m->unk48);
}
void arena_move_missiles(void)
{
    for (int i = 0; i < 10; i++)
        if (mlist[i].life >= -11)
            move_missile(&mlist[i]);
        else
            mlist[i].trail.visible = 0;
}
