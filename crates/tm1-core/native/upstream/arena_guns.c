#include "tm1/arena_guns.h"
#include "tm1/HDP.H"
#define BIGGER(a, b) (abs(a) < abs(b) ? (b) : (a))
#define blist (arena_bullets())
s32 CAR_HD(VECTOR3* pt, s32 owner, s32 flag)
{
    s32 i;
    s32 np;
    s32 nai;
    Cs* cs;

    np = GetNumPlayers();
    i = 0;
    nai = GetNumAICars();
    for (i = 0; i < np; i++) {
        cs = GetPlayerCs3D(i);
        if (cs->unkC0 != owner) {
            if (abs(BIGGER(pt->vx - cs->pos.vx, BIGGER(pt->vy - cs->pos.vy, pt->vz - cs->pos.vz)))
                >> 3 < flag + 6) {
                return cs->unkC0;
            }
        }
    }
    for (i = 0; i < nai; i++) {
        cs = GetAICs3D(i);
        if (cs->unkC0 != owner) {
            if (abs(BIGGER(pt->vx - cs->pos.vx, BIGGER(pt->vy - cs->pos.vy, pt->vz - cs->pos.vz)))
                >> 3 < flag + 6) {
                return cs->unkC0;
            }
        }
    }
    return arena_hit_target(pt, owner, flag);
}
s32 find_free_bullet(s32 count)
{
    s32 i;

    for (i = 0; i < count; i++) {
        if (blist[i].life <= 0) {
            return i;
        }
    }
    return -1;
}
void create_bullet(s32 owner, VECTOR3* dir, VECTOR3* pos, s32 damage)
{
    s32 i;
    s32 big;
    s32 mag;
    VECTOR3 vel;
    Bullet* b;

    i = find_free_bullet(40);
    if (i >= 0) {
        b = &blist[i];
        big = BIGGER(dir->vx, BIGGER(dir->vy, dir->vz));
        mag = abs(big);
        if (mag <= 0) {
            mag = 1;
        }
        if (owner == 1) {
            vel.vx = dir->vx * 250 / mag;
            vel.vy = dir->vy * 250 / mag;
            vel.vz = dir->vz * 250 / mag;
            b->life = 15;
        } else {
            setVector(&vel, dir->vx * 250 / mag, dir->vy * 250 / mag, dir->vz * 250 / mag);
            if (owner >= 50) {
                b->life = 20;
            } else {
                b->life = 15;
            }
        }
        setVector(&b->prev, pos->vx, pos->vy, pos->vz);
        setVector(&b->vel, vel.vx, vel.vy, vel.vz);
        setVector(&b->pos, pos->vx, pos->vy, pos->vz);
        b->owner = owner;
        b->kind = 0;
        b->unk30 = 0;
        b->damage = damage;
        b->unk38 = 0;
    }
}
void arena_move_bullets(void)
{
    s32 i;
    s32 k;

    Bullet* b;
    HdCsHit* hit;
    VECTOR3 p;
    VECTOR3 q;

    HdCsHit empty = { 0 };
    hit = &empty;
    for (i = 0; i < 40; i++) {
        if (blist[i].life > 0) {
            b = &blist[i];
            b->life -= 1;
            setVector(&b->prev, b->pos.vx, b->pos.vy, b->pos.vz);
            b->pos.vx += b->vel.vx;
            b->pos.vy += b->vel.vy;
            b->pos.vz += b->vel.vz;
            setVector(&p, b->prev.vx, b->prev.vy, b->prev.vz);
            for (k = 3; k >= 0; k--) {
                setVector(&q, p.vx, p.vy, p.vz);
                setVector(&p, b->pos.vx - ((b->vel.vx * k) >> 2),
                    b->pos.vy - ((b->vel.vy * k) >> 2), b->pos.vz - ((b->vel.vz * k) >> 2));
                b->unk30 = CAR_HD(&p, b->owner, (b->kind >= 15) * 4);
                if (b->unk30 == b->owner) {
                    b->unk30 = 0;
                }
                if (b->unk30 == 0 && p.vz <= 0) {
                    if (shellGetCurrentLevel() != LEVEL4_CITY_PARK || b->vel.vz < 0) {
                        b->unk30 = 8;
                    }
                }
                if (k == 2 && b->unk30 == 0) {
                    hit = HdPntTest(b->owner, -1, &p, &b->unk30);
                    if (hit->tag0 == -b->owner || hit->tag0 == 0x3FD) {
                        hit->tag0 = 0;
                        b->unk30 = 0;
                    }
                }
                if (b->unk30 > 0) {
                    if (b->unk30 == 1) {
                        setVector(&q, p.vx, p.vy, p.vz);
                    }
                    do_simple_spark(&q);
                    if (arena_rand() < 20000) {
                        do_puff(&q);
                    }
                    if (b->unk30 != 8) {
                        arena_flash(&q);
                    }
                    if (b->kind >= 15) {
                        arena_effect(&q, 28);
                        q.vz = 0;
                        arena_effect(&q, 35);
                    }
                    b->life = 0;
                    arena_effect(&p, 54); /* Machine gun impact sound at the hit test point. */
                    if (bulDispatchDamage(b->unk30, hit->tag0, hit->tag1, b->damage, &b->vel, b->owner) && b->kind == 3)
                        arena_effect(&p, 55);
                    k = -1;
                }
            }
            if (b->life <= 0 && b->kind == 7) {
                arena_effect(&b->pos, 37);
                do_smoke(&b->pos);
            }
        }
    }
}
void UpdateAIWeaponPadStatus(AICar* car)
{
    s32 n;

    n = 4;
    if (car->unk40 != 0) {
        if (car->unk43 == 0) {
            n = 5;
        }
        if (car->unk44 == 0) {
            n = 3;
        }
        if (car->unk45 == 0) {
            n = 2;
        }
        if (abs(car->unk2C) < n * 136 && car->unk34 < 4800) {
            car->skid[9] = 1;
        }
        if (car->uaIndex == UA_BOSS_CAR && car->direction == DIRECTION_REAR) {
            if (abs(car->unk2C) > (((180 - n * 10) << 12) / 360)) {
                car->skid[9] = 1;
            }
        }
        if (car->weap.cur == WEAPON_HOMING_MISSILE && n < 3) {
            n += 2;
        }
        if (car->weap.cur == WEAPON_SPECIAL && n < 3
            && (car->uaIndex == UA_HUMVEE || car->uaIndex == UA_CORVETTE)) {
            n += 2;
        }
        if (abs(car->unk2C) < n * 113) {
            car->skid[10] = 1;
        } else if (car->direction == DIRECTION_REAR) {
            if (abs(car->unk2C) > (((180 - n * 10) << 12) / 360)) {
                car->skid[10] = 1;
            }
        }
        if (car->weap.cur < WEAPON_REGULAR_MAX) {
            car->weap.ammo[car->weap.cur] = 10;
        }
    }
}
void arena_update_guns(Car* car, u8 isPlayer)
{
    VECTOR3 r;
    VECTOR3 d;
    VECTOR3 v;
    u16 idx;
    CarStats* cfg;
    CarMotion* m;
    u8* sk;
    CarWeap* pad;
    CarBounce* aim;

    MATRIX* mat;

    if (isPlayer) {
        cfg = &PLAYER_CAR(car)->stats;
        m = &PLAYER_CAR(car)->motion;
        sk = PLAYER_CAR(car)->skid;
        pad = &PLAYER_CAR(car)->weap;
        aim = &PLAYER_CAR(car)->bounce;
        idx = PLAYER_CAR(car)->playerIdx + 1;

    } else {
        cfg = &AI_CAR(car)->stats;
        m = &AI_CAR(car)->motion;
        sk = AI_CAR(car)->skid;
        pad = &AI_CAR(car)->weap;
        aim = &AI_CAR(car)->bounce;
        idx = AI_CAR(car)->playerIdx + 50;
    }
    if ((s16)pad->gunDelay > 0) {
        --pad->gunDelay;
        return;
    }
    if (!isPlayer) {
        if (arena_rand() % 100 + 1 > (s16)pad->specialChance) {
            sk[9] = 0;
        }
    }
    if ((s16)cfg->unk18 > 0) {
        sk[9] = 0;
    }
    if (isPlayer && pad->gunOverheat != 0) {
        sk[9] = 0;
    }
    arena_gun_sound(isPlayer ? PLAYER_CAR(car)->playerIdx : 1 + AI_CAR(car)->playerIdx, sk[9] != 0, &m->pos);
    if (!sk[9]) {
        if (pad->gunHeat > 0) {
            pad->gunHeat = pad->gunHeat - 1;
            if (pad->gunHeat < 100) {
                pad->gunOverheat = 0;
            }
        }
        return;
    }
    {

        if (!isPlayer)
            arena_request_battle_music();
        // Keep muzzle setup together to preserve PsyQ's load scheduling.
        do {
            if (pad->unk04 != 0) {
                v.vx = -cfg->unk110;
            } else {
                v.vx = cfg->unk110;
            }
            v.vy = (s16)cfg->unk112;
            v.vz = cfg->unk114;
        } while (0);
        mat = &m->mat2;
        pad->unk04 = 1 - pad->unk04;
        mathMulTransVec(mat, &v, &d);
        d.vx = d.vx + m->pos.vx;
        d.vy = d.vy + m->pos.vy;
        d.vz = d.vz + m->pos.vz;
        r.vx = rsin(m->rot.vz - aim->unk38);
        r.vy = rsin(m->rot.vz - aim->unk38 + 1024);
        r.vz = -rsin(m->rot.vx - aim->unk30);
        create_bullet((s16)idx, &r, &d, 1);
        arena_effect(&d, 53);
        if ((isPlayer ? PLAYER_CAR(car)->uaIndex : AI_CAR(car)->uaIndex) == UA_BOSS_CAR && pad->unk04) {
            v.vx = 0;
            v.vy = -v.vy;
            mathMulTransVec(mat, &v, &d);
            d.vx += m->pos.vx;
            d.vy += m->pos.vy;
            d.vz += m->pos.vz;
            setVector(&r, -r.vx, -r.vy, -r.vz);
            s_create_bullet(idx, &r, &d, 2, 2, 20, 0);
            arena_effect(&d, 53);
        }
        pad->gunDelay = 3;
        if (!isPlayer) {
            pad->gunDelay = 5;
        } else {
            pad->gunHeat = pad->gunHeat + 6;
            if (pad->gunHeat >= 199) {
                pad->gunOverheat = 1;
            }
        }
    }
}
void s_create_bullet(
    s32 owner, VECTOR3* dir, VECTOR3* pos, s32 kind, s32 damage, s32 life, s32 arg6)
{
    s32 i;
    s32 big;
    s32 mag;
    VECTOR3 vel;
    Bullet* b;

    i = find_free_bullet(40);
    if (i >= 0) {
        b = &blist[i];
        big = BIGGER(dir->vx, BIGGER(dir->vy, dir->vz));
        mag = abs(big);
        if (mag <= 0) {
            mag = 1;
        }
        if (kind == 4) {
            mag = mag * 5 / 4;
        } else if (kind == 6) {
            mag = mag * 5 / 4;
        }
        if (kind >= 15 || kind == 5) {
            mag *= 2;
            setVector(&vel, dir->vx * 250 / mag, dir->vy * 250 / mag, dir->vz * 250 / mag);
        } else {
            setVector(&vel, dir->vx * 250 / mag, dir->vy * 250 / mag, dir->vz * 250 / mag);
        }
        setVector(&b->prev, pos->vx, pos->vy, pos->vz);
        setVector(&b->vel, vel.vx, vel.vy, vel.vz);
        setVector(&b->pos, pos->vx, pos->vy, pos->vz);
        b->life = life;
        b->owner = owner;
        b->kind = kind;
        b->unk30 = 0;
        b->damage = damage;
        b->unk38 = arg6;
    }
}
void UpdatePlayerWeaponPadStatus(PlayerCar* car)
{
    u32 start;

    if (car->skid[0x0B] != 0) {
        if (car->skid[0x0F] == 0) {
            return;
        }
        car->skid[0x0F] = 0;
        start = car->weap.cur;
        car->weap.cur = start + 1;
        while (1) {
            if (car->weap.cur >= WEAPON_MAX) {
                car->weap.cur = WEAPON_FIRE_MISSILE;
            }
            if (car->weap.ammo[car->weap.cur] > 0) {
                return;
            }
            if (car->weap.cur == start) {
                return;
            }
            car->weap.cur++;
        }
    } else if (car->skid[0x0C] != 0) {
        if (car->skid[0x0F] == 0) {
            return;
        }
        car->skid[0x0F] = 0;
        start = car->weap.cur;
        if (car->weap.cur == WEAPON_FIRE_MISSILE) {
            car->weap.cur = WEAPON_SPECIAL;
        } else {
            car->weap.cur--;
        }
        if (car->weap.ammo[car->weap.cur] > 0) {
            return;
        }
        while (1) {
            if (car->weap.cur == start) {
                return;
            }
            if (car->weap.cur == WEAPON_FIRE_MISSILE) {
                car->weap.cur = WEAPON_SPECIAL;
            } else {
                car->weap.cur--;
            }
            if (car->weap.ammo[car->weap.cur] > 0) {
                return;
            }
        }
    } else {
        car->skid[0x0F] = 1;
    }
}

// FireSpecials in INTERACT.c. Projectile, damage, life, charge cost and cooldown.
typedef struct {
    s32 projectile, damage, life, cost, delay;
} ProjectileSpecial;
static ProjectileSpecial playerProjectileSpecial(s32 kind)
{
    switch (kind) {
    case 10:
        return (ProjectileSpecial) { 15, 30, 30, 7, 20 };
    case 30:
        return (ProjectileSpecial) { 4, 6, 12, 5, 10 };
    case 60:
        return (ProjectileSpecial) { 6, 15, 15, 3, 10 };
    case 80:
        return (ProjectileSpecial) { 21, 40, 20, 9, 20 };
    case 90:
        return (ProjectileSpecial) { 19, 15, 30, 4, 10 };
    default:
        return (ProjectileSpecial) { 17, 12, 30, 4, 20 };
    }
}
s32 arena_player_special_cost(s32 kind)
{
    if (kind == UA_HUMVEE || kind == UA_MAD_MAX)
        return 7;
    if (kind == UA_CORVETTE || kind == UA_BOSS_CAR)
        return 5;
    if (kind == UA_LOW_RIDER)
        return 5;
    return (kind == UA_POLICE_CAR || kind == UA_MONSTER_TRUCK) ? 6
                                                               : playerProjectileSpecial(kind).cost;
}
static VECTOR3 special_direction(const CarMotion* motion)
{
    return (VECTOR3){rsin(motion->rot.vz) >> 6, rsin(motion->rot.vz + 1024) >> 6,
        -rsin(motion->rot.vx) >> 6};
}

static void launch_special_flame(int owner, const CarMotion* motion, VECTOR3 dir)
{
    Cs* cs = owner < 50 ? GetPlayerCs3D(owner - 1) : GetAICs3D(owner - 50);
    VECTOR3 origin = motion->pos;
    if (arena_launch_flame(cs, 1)) {
        origin = (VECTOR3){(u32)cs->pos.vx + dir.vx, (u32)cs->pos.vy + dir.vy, (u32)cs->pos.vz + dir.vz};
        arena_record_effect(&origin, 11);
    }
    arena_special_effect(&origin, owner);
}

static void fireProjectileSpecial(int kind, int owner, CarStats* stats, CarMotion* motion, CarWeap* pad, u8* flags)
{
    ProjectileSpecial special = playerProjectileSpecial(kind);
    if (pad->ammo[WEAPON_SPECIAL] >= special.cost) {
        VECTOR3 local = { 0, stats->unk112, stats->unk114 }, d;
        mathMulTransVec(&motion->mat2, &local, &d);
        d.vx += motion->pos.vx;
        d.vy += motion->pos.vy;
        d.vz += motion->pos.vz;
        VECTOR3 dir = special_direction(motion);
        if (kind == 30)
            d.vz -= 16;
        s_create_bullet(owner, &dir, &d, special.projectile, special.damage, special.life, 0);
        pad->ammo[WEAPON_SPECIAL] -= special.cost;
        arena_special_effect(&d, owner);
    }
    pad->fireDelay = special.delay;
    flags[9] = 1;
}

static int launch_taser(int player, int slot)
{
    ArenaTaser* tasers = arena_tasers();
    for (int i = 0; i < 3; i++) {
        if (tasers[i].life > 0)
            continue;
        Cs* owner = player ? GetPlayerCs3D(slot) : GetAICs3D(slot);
        VECTOR3* origin = player ? &GetPlayerInfo(slot)->motion.pos : &GetAICarInfo(slot)->motion.pos;
        int best = 81;
        tasers[i].target = -1;
        int players = GetNumPlayers();
        for (int j = -players; j < GetNumAICars(); j++) {
            Cs* other = j < 0 ? GetPlayerCs3D(j + players) : GetAICs3D(j);
            if (other == owner || (j >= 0 && GetAICarInfo(j)->stats.unk40 <= 0))
                continue;
            int distance = abs(BIGGER(origin->vx - other->pos.vx,
                BIGGER(origin->vy - other->pos.vy, origin->vz - other->pos.vz))) >> 3;
            if (distance < best) {
                best = distance;
                tasers[i].target = other->unkC0;
            }
        }
        tasers[i].life = 10;
        tasers[i].owner = owner->unkC0;
        return 1;
    }
    return 0;
}

static void firePoliceSpecial(CarWeap* weapon, u8* flags, int player, int slot)
{
    if (weapon->ammo[WEAPON_SPECIAL] >= 6 && launch_taser(player, slot)) {
        weapon->ammo[WEAPON_SPECIAL] -= player ? 6 : 3;
        VECTOR3* origin = player ? &GetPlayerInfo(slot)->motion.pos : &GetAICarInfo(slot)->motion.pos;
        arena_special_effect(origin, player ? 1 + slot : 50 + slot);
    }
    weapon->fireDelay = 20;
    flags[9] = 1;
}

static void fireBossSpecial(int owner, CarStats* stats, CarMotion* motion, CarWeap* weapon, u8* flags)
{
    const int cost = arena_player_special_cost(UA_BOSS_CAR);
    if (weapon->ammo[WEAPON_SPECIAL] >= cost) {
        arena_special_missiles(UA_BOSS_CAR, owner, stats, motion, weapon, flags);
        VECTOR3 mount = { (s16)stats->dropPower, 0, stats->unk70 + 8 }, pos;
        mathMulTransVec(&motion->mat2, &mount, &pos);
        pos.vx += motion->pos.vx;
        pos.vy += motion->pos.vy;
        pos.vz += motion->pos.vz;
        VECTOR3 dir = special_direction(motion);
        s_create_bullet(owner, &dir, &pos, 19, 10, 30, 0);
        int player = owner < 50;
        int slot = owner - (player ? 1 : 50);
        launch_taser(player, slot);
        launch_special_flame(owner, motion, dir);
        weapon->ammo[WEAPON_SPECIAL] -= cost;
    }
    weapon->fireDelay = 20;
    flags[9] = 1;
}
void arena_move_tasers(void)
{
    ArenaTaser* tasers = arena_tasers();
    for (int i = 0; i < 3; i++) {
        tasers[i].visible = tasers[i].life > 0;
        if (!tasers[i].visible)
            continue;
        VECTOR3 origin
            = tasers[i].owner < 50 ? GetPlayerInfo(tasers[i].owner - 1)->motion.pos : GetAICarInfo(tasers[i].owner - 50)->motion.pos,
            end;
        if (tasers[i].target == 1 || tasers[i].target == 2)
            end = GetPlayerInfo(tasers[i].target - 1)->motion.pos;
        else if (tasers[i].target >= 50)
            end = GetAICarInfo(tasers[i].target - 50)->motion.pos;
        else {
            end.vx = origin.vx + (rsin(tasers[i].life * 400 + 1024) >> 4);
            end.vy = origin.vy + (rsin(tasers[i].life * 400) >> 4);
            end.vz = shellGetCurrentLevel() == LEVEL6_ROOFTOPS ? origin.vz - 40 : 0;
        }
        VECTOR3* pt = tasers[i].points;
        pt[0] = origin;
        pt[0].vz += 30;
        pt[15] = end;
        for (int j = 1; j < 15; j++) {
            pt[j].vx = origin.vx + (end.vx - origin.vx) * j / 16;
            pt[j].vy = origin.vy + (end.vy - origin.vy) * j / 16;
            pt[j].vz = origin.vz + 30 + (end.vz - origin.vz) * j / 16;
            pt[j].vz -= 32 - (arena_rand() & 63);
            pt[j].vy -= 8 - (arena_rand() & 15);
            pt[j].vx -= 8 - (arena_rand() & 15);
        }
        if (arena_rand() & 1)
            do_puff(&pt[15]);
        else
            arena_flash(&pt[15]);
        do_simple_spark(&pt[(arena_rand() & 7) + 8]);
        do_simple_spark(&pt[arena_rand() & 15]);
        if (--tasers[i].life == 0 && tasers[i].target >= 0)
            carTakeHit(tasers[i].target, 8, NULL, 0);
    }
}
#define dropweapon (arena_drops())
s32 FindFreeDropWeap(void)
{
    s32 i;

    for (i = 0; i < 12; i++) {
        if (dropweapon[i].life <= 0) {
            return i;
        }
    }
    return -1;
}

static void launchDrop(
    CarStats* stats, CarMotion* motion, CarTire* tires, CarWeap* pad, u8* flags, int owner)
{
    int i = FindFreeDropWeap();
    if (i < 0)
        return;
    DropWeap* d = &dropweapon[i];
    d->life = 400;
    d->kind = pad->cur;
    d->idx = owner;
    switch (d->kind) {
    case WEAPON_OIL: d->type = HIT_OIL; break;
    case WEAPON_BOMB: d->type = HIT_BOMB; break;
    case WEAPON_SPIKE: d->type = HIT_SPIKE; break;
    default: d->type = HIT_CATAPULT; break;
    }
    d->power = tires[2].unk22 < tires[3].unk22 ? tires[2].unk22 : tires[3].unk22;
    VECTOR3 local = { (s16)stats->dropPower, 0, 0 };
    mathMulTransVec(&motion->mat2, &local, &d->pos);
    d->pos.vx += motion->pos.vx;
    d->pos.vy += motion->pos.vy;
    d->pos.vz += motion->pos.vz;
    pad->fireDelay = 20;
    flags[9] = 1;
    pad->ammo[pad->cur]--;
    pad->totalAmmo--;
    arena_drop_launch_sound(&d->pos, d->kind);
}
void arena_move_drops(void)
{
    for (int i = 0; i < 12; i++) {
        DropWeap* d = &dropweapon[i];
        d->drawn_kind = 0;
        if (d->life <= 0)
            continue;
        if (d->power < d->pos.vz)
            d->pos.vz = d->pos.vz - 12 >= d->power ? d->pos.vz - 12 : d->power;
        d->drawn_kind = d->kind;
        d->drawn_pos = d->pos;
        --d->life;
        d->pos.vz += 30;
        int hit = CAR_HD(&d->pos, d->life >= 360 ? d->idx : 0, 2);
        d->pos.vz -= 30;
        if (hit && hit != 8) {
            arena_weapon_impact_sound(&d->pos, d->kind);
            if (d->kind == WEAPON_SPIKE)
                do_smoke(&d->pos);
            if (d->kind == WEAPON_BOMB) {
                VECTOR3 above = d->pos;
                above.vz += 50;
                arena_effect(&above, 37);
                arena_effect(&d->pos, 37);
            } else
                for (int j = 0; j < 6; j++) {
                    VECTOR3 p = d->pos;
                    p.vx += (arena_rand() & 63) - 32;
                    p.vy += (arena_rand() & 63) - 32;
                    do_puff(&p);
                }
            if (d->kind != WEAPON_OIL)
                d->life = 0;
            VECTOR3 v = { 0, 0, 100 };
            if (d->kind == WEAPON_SPIKE)
                bulDispatchDamage(hit, 0, 0, 8, &v, -d->kind);
            else if (d->kind == WEAPON_BOMB)
                bulDispatchDamage(hit, 0, 0, 15, &v, -d->kind);
            bulDispatchDamage(hit, 0, 0, d->type, &v, -d->kind);
        }
    }
}
void arena_update_weapons(Car* car, u8 player)
{
    PlayerCar* p = PLAYER_CAR(car);
    AICar* ai = AI_CAR(car);
    CarWeap* pad = player ? &p->weap : &ai->weap;
    CarStats* stats = player ? &p->stats : &ai->stats;
    u8* sk = player ? p->skid : ai->skid;
    if (player)
        UpdatePlayerWeaponPadStatus(p);
    else
        UpdateAIWeaponPadStatus(ai);
    if (pad->reset) {
        for (int i = 0; i < WEAPON_AMMO_MAX; i++)
            pad->ammo[i] = 1;
        pad->ammo[WEAPON_SINGING_MISSILE] = 0;
        pad->ammo[WEAPON_NONE] = 0;
        pad->ammo[WEAPON_SPECIAL] = 20;
        pad->unk46 = 2;
    } else if (pad->timer++ >= 80) {
        pad->timer = 0;
        if (pad->ammo[WEAPON_SPECIAL] < 20)
            pad->ammo[WEAPON_SPECIAL]++;
    }
    arena_update_guns(car, player);
    if ((s16)pad->fireDelay > 0) {
        pad->fireDelay--;
        return;
    }
    if (pad->cur == WEAPON_NONE || (s16)stats->unk18 > 0)
        sk[10] = 0;
    if (!sk[10])
        return;
    if (!player && pad->cur != WEAPON_SPECIAL)
        pad->ammo[pad->cur] = 5;
    if (!pad->ammo[pad->cur])
        return;
    if (!player)
        arena_request_battle_music();
    switch (pad->cur) {
    case WEAPON_SPECIAL: {
        int kind = player ? p->uaIndex : ai->uaIndex;
        int owner = player ? p->playerIdx + 1 : ai->playerIdx + 50;
        u8* flags = player ? p->flags : ai->flags;
        CarMotion* motion = player ? &p->motion : &ai->motion;
        if (kind == UA_BOSS_CAR)
            fireBossSpecial(owner, stats, motion, pad, flags);
        else if (kind == UA_HUMVEE || kind == UA_CORVETTE || kind == UA_MAD_MAX)
            arena_special_missiles(kind, owner, stats, motion, pad, flags);
        else if (kind == UA_LOW_RIDER) {
            if (pad->ammo[WEAPON_SPECIAL] >= 5) {
                launch_special_flame(owner, motion, special_direction(motion));
                pad->ammo[WEAPON_SPECIAL] -= 5;
            }
            pad->fireDelay = 20;
            flags[9] = 1;
        } else if (kind == UA_MONSTER_TRUCK) {
            pad->fireDelay = 20;
            flags[9] = 1;
        } else if (kind == UA_POLICE_CAR)
            firePoliceSpecial(pad, flags, player, player ? p->playerIdx : ai->playerIdx);
        else
            fireProjectileSpecial(kind, owner, stats, motion, pad, flags);
        break;
    }
    case WEAPON_FIRE_MISSILE:
    case WEAPON_HOMING_MISSILE:
    case WEAPON_FREEZE_MISSILE:
    case WEAPON_POWER_MISSILE:
    case WEAPON_SINGING_MISSILE:
    case WEAPON_REAR_MISSILE:
        arena_launch_missile(
            stats, player ? &p->motion : &ai->motion, pad, player ? p->flags : ai->flags, player ? 1 + p->playerIdx : 50 + ai->playerIdx);
        break;
    case WEAPON_FLAMETHROWER: {
        Cs* cs = player ? GetPlayerCs3D(p->playerIdx) : GetAICs3D(ai->playerIdx);
        arena_record_effect(player ? &p->motion.pos : &ai->motion.pos, 11);
        if (arena_launch_flame(cs, 0)) {
            pad->fireDelay = 20;
            pad->ammo[WEAPON_FLAMETHROWER]--;
        }
        break;
    }
    case WEAPON_OIL:
    case WEAPON_SPIKE:
    case WEAPON_CATAPULT:
    case WEAPON_BOMB:
        launchDrop(stats, player ? &p->motion : &ai->motion, player ? p->tires : ai->tires, pad,
            player ? p->flags : ai->flags, player ? 1 + p->playerIdx : 50 + ai->playerIdx);
        break;
    }
}
typedef struct {
    u32 address;
    s16 special_chance, fore_chance, aft_chance;
    s32 front[4], rear[6];
} AIWeaponProfile;

// AICARUPD uses the original 32-bit profile addresses inside each car record.
static const AIWeaponProfile ai_weapons[] = {
    [UA_BOSS_CAR / 10] = { 0x80171260, 100, 75, 75, { 15, 15, 35, 35 }, { 20, 10, 25, 20, 10, 15 } },
    [UA_ICE_CREAM_TRUCK / 10] = { 0x80171080, 35, 60, 90, { 33, 0, 65, 2 }, { 0, 90, 10, 0, 0, 0 } },
    [UA_TAXI_CAB / 10] = { 0x801710a8, 90, 10, 75, { 70, 0, 30, 0 }, { 10, 0, 0, 90, 0, 0 } },
    [UA_SEMI_TRUCK / 10] = { 0x801710d0, 35, 70, 10, { 20, 80, 0, 0 }, { 20, 70, 10, 0, 0, 0 } },
    [UA_MONSTER_TRUCK / 10] = { 0x80171120, 50, 20, 50, { 10, 90, 0, 0 }, { 0, 80, 0, 20, 0, 0 } },
    [UA_POLICE_CAR / 10] = { 0x80171170, 35, 25, 50, { 10, 90, 0, 0 }, { 0, 20, 0, 0, 80, 0 } },
    [UA_LAMBORGHINI / 10] = { 0x80171238, 95, 30, 80, { 95, 0, 0, 5 }, { 90, 0, 10, 0, 0, 0 } },
    [UA_HUMVEE / 10] = { 0x801711e8, 60, 60, 60, { 90, 0, 5, 5 }, { 0, 0, 100, 0, 0, 0 } },
    [UA_HARLEY / 10] = { 0x801711c0, 35, 80, 60, { 80, 20, 0, 0 }, { 0, 0, 10, 0, 0, 90 } },
    [UA_DUNE_BUGGY / 10] = { 0x80171148, 50, 50, 90, { 90, 5, 5, 0 }, { 50, 20, 0, 0, 0, 30 } },
    [UA_LOW_RIDER / 10] = { 0x801710f8, 75, 80, 80, { 20, 20, 0, 0 }, { 35, 5, 60, 0, 0, 0 } },
    [UA_CORVETTE / 10] = { 0x80171210, 90, 30, 10, { 100, 0, 0, 0 }, { 0, 0, 0, 100, 0, 0 } },
    [UA_MAD_MAX / 10] = { 0x80171198, 80, 50, 20, { 100, 0, 0, 0 }, { 20, 0, 20, 0, 30, 30 } },
};

void arena_ai_weapon_profile(AICar* car)
{
    const AIWeaponProfile* p = &ai_weapons[car->uaIndex / 10];
    car->weap.foreProfile = p->address;
    car->weap.aftProfile = p->address + sizeof(p->front);
    car->weap.specialChance = p->special_chance;
    car->weap.foreChance = p->fore_chance;
    car->weap.aftChance = p->aft_chance;
    if (uaGetDifficulty() == DIFFICULTY_EASY) {
        car->weap.specialChance = p->special_chance >= 25 ? p->special_chance - 15 : 10;
        car->weap.foreChance = p->fore_chance >= 20 ? p->fore_chance - 10 : 10;
        car->weap.aftChance = p->aft_chance >= 20 ? p->aft_chance - 10 : 10;
    } else if (uaGetDifficulty() == DIFFICULTY_HARD) {
        car->weap.specialChance += 20;
        car->weap.foreChance = p->fore_chance < 85 ? p->fore_chance + 10 : 95;
        car->weap.aftChance = p->aft_chance < 85 ? p->aft_chance + 10 : 95;
    }
}

void AICarChooseForeWeapon(AICar* car)
{
    s32 sum = 0;
    s16 roll = arena_rand() % 100;
    s32 i = WEAPON_FIRE_MISSILE;
    const s32* p = ai_weapons[car->uaIndex / 10].front;

    do {
        sum += *p;
        if (roll < sum) {
            car->weap.cur = i;
            return;
        }
        i++;
        p++;
    } while (i <= WEAPON_POWER_MISSILE);
}
void AICarChooseAftWeapon(AICar* car)
{
    s32 sum = 0;
    s16 roll = arena_rand() % 100;
    const s32* probs = ai_weapons[car->uaIndex / 10].rear;
    s32 i = WEAPON_REAR_MISSILE;

    do {
        sum += probs[i - WEAPON_REAR_MISSILE];
        if (roll < sum) {
            car->weap.cur = i;
            return;
        }
        i++;
    } while (i <= WEAPON_SPIKE);
}
static u8 AIInSpecialParameters(AICar* car)
{
    s32 value;

    value = car->uaIndex;
    switch (value) {
    case UA_ICE_CREAM_TRUCK:
    case UA_TAXI_CAB:
    case UA_SEMI_TRUCK:
    case UA_LAMBORGHINI:
    case UA_HARLEY:
    case UA_DUNE_BUGGY:
    case UA_CORVETTE:
        return car->direction == DIRECTION_FRONT;
    case UA_BOSS_CAR:
        if (car->direction == DIRECTION_FRONT) {
            return 1;
        }
        if (car->unk44 != 0) {
            return 1;
        }
        return car->direction == DIRECTION_REAR;
    default:
        value = car->direction;
        if (value != DIRECTION_FRONT) {
            return 0;
        }
        return car->unk44 != 0;
    case UA_POLICE_CAR:
        return car->unk44 != 0;
    }
}

void AIPickAttackWeapon(AICar* car)
{
    s16 lvl;

    car->weap.cur = WEAPON_NONE;
    lvl = car->weap.ammo[WEAPON_SPECIAL];
    if (lvl >= 6 && AIInSpecialParameters(car)) {
        if (arena_rand() % 24 < car->weap.ammo[WEAPON_SPECIAL]) {
            car->weap.cur = WEAPON_SPECIAL;
            return;
        }
    }
    switch (car->direction) {
    case DIRECTION_FRONT:
        if (arena_rand() % 100 + 1 < (s16)car->weap.foreChance) {
            AICarChooseForeWeapon(car);
        } else {
            car->weap.cur = WEAPON_NONE;
        }
        break;
    case DIRECTION_REAR:
        if (arena_rand() % 100 + 1 < (s16)car->weap.aftChance) {
            AICarChooseAftWeapon(car);
        } else {
            car->weap.cur = WEAPON_NONE;
        }
        break;
    }
}
void arena_update_ai_guns(AICar* car)
{
    if (!arena_bullets())
        return;
    if (car->unk40 != 0) {
        if ((s16)car->weap.fireDelay < 2) {
            AIPickAttackWeapon(car);
            if (car->weap.cur == WEAPON_NONE) {
                car->weap.fireDelay = arena_rand() % 20 + 10;
            }
        }
        arena_update_weapons(car, 0);
    } else {
        car->weap.cur = WEAPON_NONE;
    }
}
