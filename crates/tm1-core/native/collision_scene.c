#include "tm1/arena_collision.h"
#include "tm1/arena_guns.h"
#include <stdbool.h>

// Native nodes contain host pointers. Every pointer slot in the JP data is
// resolved separately; only fixed-width scalar records are copied.
typedef struct CollisionScene {
    const u8* bytes;
    size_t size;
    u32 base;
    DbNode** nodes;
    u8* visiting;
    size_t slots;
    Cs world, taxi, opponents[8];
    Cs* first_car;
    int opponent_count;
    DbSwitch* stands[10];
    u16 stand_slots, active_stands;
    int regen_timer;
    Bullet bullets[40];
    Missile missiles[10];
    DropWeap drops[12];
    ArenaTaser tasers[3];
    struct {
        Cs* owner;
        int forward, age, life;
    } flames[5];
    struct {
        VECTOR3 pos, vel;
        int frame, life;
    } bursts[80];
    struct {
        VECTOR3 pos, vel;
        int life, frame, variant;
    } fragments[20];
    PotHole temporary_hole;
    int pothole_fields;
    bool has_temporary_hole;
    int battle_wait;
    DbTrans* pickups[50];
    u32 pickup_addr[50];
    int num_pickups, pickup_timer;
    Cs missile_cs[10];
    s32 effects[80][5];
    size_t next_effect;
} CollisionScene;

static bool load(CollisionScene* s, u32 addr, size_t offset, void* out, size_t n)
{
    if (addr < s->base || (addr & 3))
        return false;
    size_t i = (size_t)(addr - s->base) + offset;
    if (i > s->size || n > s->size - i)
        return false;
    memcpy(out, s->bytes + i, n);
    return true;
}
static DbNode* decode(CollisionScene*, u32, unsigned);
static bool children(
    CollisionScene* s, u32 addr, size_t offset, DbNode** out, size_t count, unsigned depth)
{
    for (size_t i = 0; i < count; i++) {
        u32 ptr;
        if (!load(s, addr, offset + 4 * i, &ptr, 4) || !(out[i] = decode(s, ptr, depth + 1)))
            return false;
    }
    return true;
}

static DbNode* decode(CollisionScene* s, u32 addr, unsigned depth)
{
    if (depth >= 64 || addr < s->base || (addr & 3))
        return NULL;
    size_t slot = (addr - s->base) / 4;
    if (slot >= s->slots || s->visiting[slot])
        return NULL;
    if (s->nodes[slot])
        return s->nodes[slot];
    DbNode* n = calloc(1, sizeof(*n));
    if (!n)
        return NULL;
    s->nodes[slot] = n;
    s->visiting[slot] = 1;
    if (!load(s, addr, 0, &n->h, 4))
        return NULL;
    switch (n->h.op) {
    case DB_OP_OBJECT:
        if (!load(s, addr, 16, &n->obj.nPoly, 28) || !load(s, addr, 44, &n->obj.box, sizeof(HdBox)))
            return NULL;
        break;
    case DB_OP_GROUP:
        if (!load(s, addr, 4, &n->grp.center, 20)
            || !children(s, addr, 24, n->grp.child, n->grp.nChild, depth))
            return NULL;
        break;
    case DB_OP_LOD:
        if (!load(s, addr, 4, &n->lod.center, 16) || (u32)n->lod.nRec > 256)
            return NULL;
        for (int i = 0; i < n->lod.nRec; i++) {
            u32 ptr;
            DbLodRec* r = calloc(1, sizeof(*r));
            n->lod.rec[i] = r;
            if (!r || !load(s, addr, 20 + i * 4, &ptr, 4) || !load(s, ptr, 0, &r->far, 8)
                || !children(s, ptr, 8, &r->child, 1, depth))
                return NULL;
        }
        break;
    case DB_OP_ROT:
        if (!load(s, addr, 4, &n->rot.mat, 36)
            || !children(s, addr, 40, n->rot.child, n->rot.nChild, depth))
            return NULL;
        break;
    case DB_OP_TRANS:
        if (!load(s, addr, 4, &n->trans.ofs, 16)
            || !children(s, addr, 20, n->trans.child, n->trans.nChild, depth))
            return NULL;
        if (UA_IS_PICKUP(n->h.kind) && n->trans.sub && n->trans.sub <= 50 && n->trans.nChild == 1) {
            int i = n->trans.sub - 1;
            if (!s->pickups[i])
                s->num_pickups++;
            s->pickups[i] = &n->trans;
            s->pickup_addr[i] = addr;
        }
        break;
    case DB_OP_ANIM:
        if (!load(s, addr, 4, &n->anim.sub, 12) || n->anim.cur >= n->anim.nChild
            || !children(s, addr, 16, n->anim.child, n->anim.nChild, depth))
            return NULL;
        break;
    case DB_OP_SWITCH:
        if (!load(s, addr, 4, &n->sw.sub, 4) || n->sw.sel >= n->sw.nChild
            || !children(s, addr, 8, n->sw.child, n->sw.nChild, depth))
            return NULL;
        if (n->h.kind == 920 && n->sw.sub >= 1 && n->sw.sub <= 10 && n->sw.nChild == 2) {
            int slot = n->sw.sub - 1;
            s->stands[slot] = &n->sw;
            s->stand_slots |= 1u << slot;
            s->active_stands = s->stand_slots;
        }
        break;
    case DB_OP_BOX:
        if (!load(s, addr, 0, &n->box, sizeof(n->box)))
            return NULL;
        break;
    case DB_OP_VOLUME:
        if (!load(s, addr, 0, &n->vol, sizeof(n->vol)) || n->vol.nplane > 6 || n->vol.code > 5)
            return NULL;
        break;
    case DB_OP_CTRL:
        if (!load(s, addr, 4, &n->ctrl.flags, 8))
            return NULL;
        for (int i = 0; i < 2; i++) {
            u32 ptr;
            if (!load(s, addr, 12 + i * 4, &ptr, 4))
                return NULL;
            if (ptr) {
                DbNode* child = decode(s, ptr, depth + 1);
                if (!child)
                    return NULL;
                if (i)
                    n->ctrl.childIn = child;
                else
                    n->ctrl.childOut = child;
            }
        }
        for (int i = 0; i < n->ctrl.nBox; i++) {
            u32 ptr;
            n->ctrl.box[i] = malloc(sizeof(HdBox));
            if (!n->ctrl.box[i] || !load(s, addr, 20 + i * 4, &ptr, 4)
                || !load(s, ptr, 0, n->ctrl.box[i], sizeof(HdBox)))
                return NULL;
        }
        break;
    case DB_OP_UNK_6:
    case DB_OP_UNK_10:
        break;
    default:
        return NULL;
    }
    s->visiting[slot] = 0;
    return n;
}
static CollisionScene* current;
void tm1_scene_free(CollisionScene* s)
{
    if (!s)
        return;
    if (s->nodes)
        for (size_t i = 0; i < s->slots; i++) {
            DbNode* n = s->nodes[i];
            if (!n)
                continue;
            if (n->h.op == DB_OP_LOD)
                for (int j = 0; j < n->lod.nRec && j < 256; j++)
                    free(n->lod.rec[j]);
            if (n->h.op == DB_OP_CTRL)
                for (int j = 0; j < n->ctrl.nBox; j++)
                    free(n->ctrl.box[j]);
            free(n);
        }
    free(s->nodes);
    free(s->visiting);
    free(s);
}
CollisionScene* tm1_scene_load(
    const u8* data, size_t size, u32 base, u32 world, u32 taxi, const u32* opponents, size_t count)
{
    if (!count || count > 8)
        return NULL;
    CollisionScene* s = calloc(1, sizeof(*s));
    if (!s)
        return NULL;
    s->bytes = data;
    s->size = size;
    s->base = base;
    s->slots = size / 4;
    s->nodes = calloc(s->slots, sizeof(DbNode*));
    s->visiting = calloc(s->slots, 1);
    if (!s->nodes || !s->visiting || !(s->world.epNode = decode(s, world, 0))
        || !(s->taxi.epNode = decode(s, taxi, 0))) {
        tm1_scene_free(s);
        return NULL;
    }
    for (int i = 0; i < 10; i++) {
        s->missiles[i].cs = &s->missile_cs[i];
        s->missiles[i].index = i;
        s->missiles[i].life = -15;
        s->missile_cs[i].unkC0 = 200 + i;
    }
    s->opponent_count = count;
    s->world.unkC0 = 8;
    s->taxi.unkC0 = 1;
    u32 roots;
    if (!load(s, base, 20, &roots, sizeof(roots)) || roots > (size - 24) / 4)
        goto invalid;
    // DB reads roots in directory order. csAddToCsList prepends each model.
    for (u32 i = 0; i < roots; i++) {
        u32 root;
        if (!load(s, base, 24 + 4 * i, &root, sizeof(root)))
            goto invalid;
        for (size_t j = 0; j <= count; j++) {
            if (root != (j ? opponents[j - 1] : taxi))
                continue;
            Cs* cs = j ? &s->opponents[j - 1] : &s->taxi;
            if (cs->drawMode || !(cs->epNode = decode(s, root, 0)))
                goto invalid;
            cs->unkC0 = j ? 49 + j : 1;
            cs->unk0C = cs->drawMode = 1;
            cs->next = s->first_car;
            s->first_car = cs;
        }
    }
    if (!s->taxi.drawMode)
        goto invalid;
    for (size_t i = 0; i < count; i++)
        if (!s->opponents[i].drawMode)
            goto invalid;
    s->regen_timer = -1;
    s->bytes = NULL;
    return s;
invalid:
    tm1_scene_free(s);
    return NULL;
}

Bullet* arena_bullets(void)
{
    return current ? current->bullets : NULL;
}
// UA.c PlayGame runs this once before updating the AI cars in slot order.
void tm1_schedule_battle(s32* waited, AICar* cars, s16 count, s16 engaged, s16 fields)
{
    if (count == 0 || engaged >= (count == 1 ? 1 : 2)) {
        *waited = 0;
        for (int i = 0; i < count; i++)
            cars[i].chosen = 0;
        return;
    }
    *waited += fields;
    if (*waited <= 180)
        return;
    AICar* best = NULL;
    s32 priority = -1;
    for (int i = 0; i < count; i++) {
        AICar* candidate = &cars[i];
        if (candidate->stats.unk40 <= 0)
            continue;
        CarRoute* route = &candidate->profile.route[candidate->unk02];
        if (!best || (candidate->unk01 && priority < route->prio)) {
            best = candidate;
            priority = route->prio;
        }
    }
    if (best) {
        best->chosen = 1;
        best->unk01 = 0;
        best->routeVal = best->profile.route[best->unk02].val;
    }
    *waited = 0;
}

void arena_schedule_battle(AICar* cars, s16 count, s16 engaged, s16 fields)
{
    if (current)
        tm1_schedule_battle(&current->battle_wait, cars, count, engaged, fields);
}
DropWeap* arena_drops(void)
{
    return current->drops;
}
ArenaTaser* arena_tasers(void)
{
    return current->tasers;
}
Missile* arena_missiles(void)
{
    return current->missiles;
}
u8 health_stand_active(s32 id)
{
    return current && id >= 1 && id <= 10 && (current->active_stands & (1u << (id-1)));
}

u32 tm1_stand_state(const CollisionScene* scene)
{
    u32 state = scene->active_stands;
    for (int i = 0; i < 10; i++)
        if (scene->stands[i] && scene->stands[i]->sel)
            state |= 1u << (16+i);
    return state;
}

u8 tm1_recharge_player(PlayerCar* player)
{
    int id = player->standId;
    if (!player->flags[22] || !health_stand_active(id))
        return 0;
    DbSwitch* model = current->stands[id-1];
    arena_effect(&player->motion.pos, 26);
    model->sel = arena_rand() & 1;
    int hp = player->stats.unk40 + player->stats.healthRate;
    player->stats.unk40 = hp;
    if (hp > player->stats.unk44)
        player->stats.unk40 = player->stats.unk44;
    else if (hp > player->stats.healthCap)
        player->stats.unk40 = player->stats.healthCap;
    else
        return 1;
    current->active_stands &= ~(1u << (id-1));
    model->sel = 1;
    return 0;
}

void tm1_regenerate_stands(void)
{
    if (!current->stand_slots || current->active_stands)
        return;
    if (current->regen_timer < 0) {
        current->regen_timer = 4000;
    } else if (current->regen_timer == 0) {
        current->active_stands = current->stand_slots;
        current->regen_timer = -1;
        for (int i = 0; i < 10; i++)
            if (current->stands[i])
                current->stands[i]->sel = 0;
    } else {
        current->regen_timer--;
    }
}

void tm1_reset_projectiles(CollisionScene* s)
{
    memset(s->bullets, 0, sizeof s->bullets);
    memset(s->effects, 0, sizeof s->effects);
    s->next_effect = 0;
    s->has_temporary_hole = false;
    s->pothole_fields = 0;
    memset(&s->temporary_hole, 0, sizeof(PotHole));
    memset(s->fragments, 0, sizeof s->fragments);
    memset(s->drops, 0, sizeof s->drops);
    memset(s->flames, 0, sizeof s->flames);
    memset(s->bursts, 0, sizeof s->bursts);
    memset(s->tasers, 0, sizeof s->tasers);
    s->battle_wait = 0;
    s->active_stands = s->stand_slots;
    s->regen_timer = -1;
    for (int i = 0; i < 10; i++)
        if (s->stands[i])
            s->stands[i]->sel = 0;
    s->pickup_timer = 0;
    for (int i = 0; i < 50; i++)
        if (s->pickups[i])
            s->pickups[i]->nChild = 1;
    for (int i = 0; i < 10; i++) {
        s->missiles[i].life = -15;
        s->missiles[i].target = NULL;
        s->missiles[i].trail.visible = 0;
    }
}
void tm1_use_scene(CollisionScene* s)
{
    if (current) {
        current->temporary_hole = potHoles[numPotHoles];
        current->has_temporary_hole = numCheckPotHoles > numPotHoles;
    }
    current = s;
    if (s)
        potHoles[numPotHoles] = s->temporary_hole;
    numCheckPotHoles = numPotHoles + (s && s->has_temporary_hole);
}
Cs* csGetWorldCs(void)
{
    return &current->world;
}
s16 GetNumAICars(void)
{
    return current ? current->opponent_count : 0;
}
Cs* csGetCsList(void)
{
    return current->first_car;
}
Cs* GetPlayerCs3D(s16 i)
{
    (void)i;
    return &current->taxi;
}
Cs* GetAICs3D(s16 i)
{
    return &current->opponents[i];
}
u8 uaIsCarCS(Cs* cs, u8* player, s16* idx)
{
    if (cs == &current->taxi) {
        *idx = 0;
        *player = 1;
        return 1;
    }
    for (s16 slot = 0; slot < current->opponent_count; slot++)
        if (cs == &current->opponents[slot]) {
            *idx = slot;
            *player = 0;
            return 1;
        }
    return 0;
}
s16 uaGetCarMatID(s16 idx, u8 player)
{
    return player ? GetPlayerCs3D(idx)->unkC0 : GetAICs3D(idx)->unkC0;
}
void tm1_contact_setup(Cs* cs, const CarMotion* motion, const CarStats* stats)
{
    cs->mat = motion->mat2;
    cs->pos = motion->pos;
    cs->nhist = 5;
    for (int i = 0; i < 5; i++) {
        cs->hist[i].vx = i == 4 ? 0 : ((i == 1 || i == 2) ? stats->unk74 / 2 : -stats->unk74 / 2);
        cs->hist[i].vy = (i == 2 || i == 3) ? stats->unk84 : stats->unk80;
        cs->hist[i].vz = 24 - stats->unk6C;
    }
}

s32 uaIsCarMatID(s16 id, s8* player, u16* idx)
{
    if (id == 1 || (id >= 50 && id < 50 + current->opponent_count)) {
        *player = id == 1;
        *idx = *player ? 0 : id - 50;
        return 1;
    }
    return 0;
}
void bulDispatchDamage(s32 id, s32 tag, s32 sub, s32 dmg, VECTOR3* dir, s32 owner)
{
    (void)tag;
    (void)sub;
    (void)owner;
    if (id == 1 || (id >= 50 && id < 50 + current->opponent_count))
        carTakeHit(id, dmg, dir, 0);
}
void InitMonsterSmash(Car* carA, u8 isPlayerA, Car* carB, u8 isPlayerB, s16 unused_mode)
{
    u8* fa;
    CarStats* sa;
    CarWeap* wa;
    u8* fb;
    CarStats* sb;
    CarMotion* mb;
    s16 idx;
    s32 uaIdx;
    Cs* cs;
    s32 dmg;
    s32 v;
    s32 qs;

    (void)unused_mode;
    if (isPlayerA) {
        fa = PLAYER_CAR(carA)->flags;
        sa = &PLAYER_CAR(carA)->stats;
        wa = &PLAYER_CAR(carA)->weap;
    } else {
        fa = AI_CAR(carA)->flags;
        sa = &AI_CAR(carA)->stats;
        wa = &AI_CAR(carA)->weap;
    }

    if (isPlayerB) {
        fb = PLAYER_CAR(carB)->flags;
        sb = &PLAYER_CAR(carB)->stats;
        mb = &PLAYER_CAR(carB)->motion;
        idx = PLAYER_CAR(carB)->playerIdx;
        uaIdx = PLAYER_CAR(carB)->uaIndex;
    } else {
        fb = AI_CAR(carB)->flags;
        sb = &AI_CAR(carB)->stats;
        mb = &AI_CAR(carB)->motion;
        idx = AI_CAR(carB)->playerIdx;
        uaIdx = AI_CAR(carB)->uaIndex;
    }

    if (wa->ammo[WEAPON_SPECIAL] < 6) {
        return;
    }
    if (fa[27]) {
        return;
    }

    arena_effect(isPlayerA ? &PLAYER_CAR(carA)->motion.pos : &AI_CAR(carA)->motion.pos, isPlayerA ? 10 : 18 + AI_CAR(carA)->playerIdx);
    wa->ammo[WEAPON_SPECIAL] = wa->ammo[WEAPON_SPECIAL] - 6;
    if (isPlayerB) {
        cs = GetPlayerCs3D(idx);
    } else {
        cs = GetAICs3D(idx);
    }
    dmg = (s16)cs->unkC0;
    carTakeHit(dmg, 12, NULL, 0);
    fb[16] = 1;
    sb->unk18 = 60;
    mb->vel.vy = 0;
    fa[27] = 1;
    fb[28] = 1;
    sa->unk30 = uaIdx;
    if (sb->unk84 < sb->unk80) {
        v = sb->unk80;
    } else {
        v = sb->unk84;
    }
    sa->unk2A = v;
    sa->unk2A += sa->unk78 / 2;
    sa->unk2C = sb->unk6C + sb->unk70;
    if (sa->unk2C > 80) {
        sa->unk2C = 80;
    }
}

// CARUPD.c CheckMonsterSmash uses the victim model position for all four tires.
static void check_smash(CarStats* st, CarMotion* motion, CarTire* tires, u8* flags,
    Cs* target, CarStats* victim, u8* victim_flags)
{
    int best = 0x7fff;
    flags[27] = 0;
    if (target) {
        VECTOR3 p = target->pos;
        for (int i = 0; i < 4; i++) {
            CarTire* tire = &tires[i];
            int dx = p.vx - tire->unk38, dy = p.vy - tire->unk3C;
            int d = arena_sqrt(dx * dx + dy * dy);
            if (d < st->unk2A) {
                flags[27] = 1;
                tire->unk20 = st->unk2C - st->unk2C * d / st->unk2A / 2;
                if (d < best)
                    best = d;
            } else {
                tire->unk20 = 0;
            }
        }
        if (flags[27] && best < st->unk2A / 2) {
            VECTOR3 pos = motion->pos;
            pos.vx += (arena_rand() & 63) - 32;
            pos.vy += (arena_rand() & 63) - 32;
            pos.vz -= (arena_rand() & 31) + 32;
            if (pos.vz <= 0)
                pos.vz = 1;
            explodeCreateFragments(arena_rand() & 3, &pos);
            switch (arena_rand() & 15) {
            case 0:
            case 1:
            case 3:
                do_smoke(&pos);
                break;
            case 2:
                arena_effect(&pos, 2);
                break;
            }
            if (victim->unk4C && !(GetFrameCount() & 3) && victim->unk40 > 0)
                victim->unk40--;
        }
        if (!flags[27])
            victim_flags[28] = 0;
    }
    if (!flags[27])
        for (int i = 0; i < 4; i++)
            tires[i].unk20 = 0;
}
void tm1_check_player_smash(PlayerCar* car, AICar* other)
{
    if (other && car->stats.unk30 != other->uaIndex)
        other = NULL;
    check_smash(&car->stats, &car->motion, car->tires, car->flags,
        other ? GetAICs3D(other->playerIdx) : NULL,
        other ? &other->stats : NULL, other ? other->flags : NULL);
}

void tm1_check_ai_smash(AICar* car)
{
    PlayerCar* player = GetPlayerInfo(0);
    if (player->uaIndex == car->stats.unk30) {
        check_smash(&car->stats, &car->motion, car->tires, car->flags,
            GetPlayerCs3D(0), &player->stats, player->flags);
        return;
    }
    for (int i = 0; i < GetNumAICars(); i++) {
        AICar* other = GetAICarInfo(i);
        if (other->uaIndex == car->stats.unk30) {
            check_smash(&car->stats, &car->motion, car->tires, car->flags,
                GetAICs3D(i), &other->stats, other->flags);
            return;
        }
    }
    check_smash(&car->stats, &car->motion, car->tires, car->flags, NULL, NULL, NULL);
}

uint64_t tm1_hidden_pickups(CollisionScene* s)
{
    uint64_t mask = 0;
    for (int i = 0; i < 50; i++)
        if (s->pickups[i] && !s->pickups[i]->nChild)
            mask |= UINT64_C(1) << i;
    return mask;
}
u32 tm1_pickup_address(CollisionScene* s, int i)
{
    return i >= 0 && i < 50 ? s->pickup_addr[i] : 0;
}
void arena_regenerate_pickups(void)
{
    if (current->pickup_timer < 250) {
        current->pickup_timer++;
        return;
    }
    current->pickup_timer = 0;
    if (current->num_pickups > 0) {
        int i = arena_rand() % current->num_pickups;
        if (current->pickups[i])
            current->pickups[i]->nChild = 1;
    }
}
void carGetPickup(Car* c, u8 player, s32 tag, s32 sub)
{
    static const s16 amounts[] = { 4, 2, 2, 2, 1, 4, 2, 2, 2, 2, 3 };
    CarWeap* pad = player ? &PLAYER_CAR(c)->weap : &AI_CAR(c)->weap;
    int cap = (s16)pad->maxAmmo - (s16)pad->totalAmmo;
    if (cap <= 0 && tag != 430 && player)
        return;
    int slot = -1;
    if (tag >= 401 && tag <= 404)
        slot = tag - 401;
    else if (tag == 405)
        slot = 5;
    else if (tag == 406)
        slot = 4;
    else if (tag >= 410 && tag <= 414)
        slot = tag - 404;
    if (slot >= 0)
        pad->ammo[slot] += cap < amounts[slot] ? cap : amounts[slot];
    else if (tag == 450)
        pad->ammo[WEAPON_SPECIAL] = 20;
    else if (tag == 430)
        pad->unk46++;
    else if (tag == 431)
        pad->unk44 += 120;
    if (sub > 0 && sub <= 50 && current->pickups[sub - 1])
        current->pickups[sub - 1]->nChild = 0;
    pad->totalAmmo = 0;
    for (int i = 0; i < WEAPON_REGULAR_MAX; i++)
        pad->totalAmmo += pad->ammo[i];
}
void UAeffectBarricade(s32 tag, s32 sub, s32 bearing)
{
    (void)tag;
    (void)sub;
    (void)bearing;
}
void arena_effect(VECTOR3* pos, s32 kind)
{
    if (!current)
        return;
    s32* e = current->effects[current->next_effect++ % 80];
    e[0] = pos->vx;
    e[1] = pos->vy;
    e[2] = pos->vz;
    e[3] = kind;
    e[4] = kind == 2 ? 7 : 2;
}
void tm1_age_effects(void)
{
    for (int i = 0; i < 80; i++)
        if (current->effects[i][4] > 0)
            current->effects[i][4]--;
}
void explodeCreateFragments(s32 count, VECTOR3* pos)
{
    if (!current)
        return;
    for (int i = 0; count > 0 && i < 20; i++) {
        if (current->fragments[i].life > 0)
            continue;
        current->fragments[i].life = 40;
        current->fragments[i].frame = 0;
        current->fragments[i].pos = *pos;
        VECTOR3* v = &current->fragments[i].vel;
        v->vx = (arena_rand() & 63) - 32;
        v->vy = (arena_rand() & 63) - 32;
        v->vz = (arena_rand() & 31) + 16;
        current->fragments[i].variant = arena_rand() & 3;
        count--;
    }
}
void arena_move_fragments(void)
{
    for (int i = 0; i < 20; i++)
        if (current->fragments[i].life > 0) {
            VECTOR3 *p = &current->fragments[i].pos, *v = &current->fragments[i].vel;
            p->vx += v->vx;
            p->vy += v->vy;
            p->vz += v->vz;
            v->vz -= 4;
            if (p->vz <= 0 && v->vz < 0) {
                if (v->vz >= -5)
                    current->fragments[i].life = 0;
                else
                    v->vz = -v->vz / 2;
            }
            if (current->fragments[i].life > 0) {
                current->fragments[i].life--;
                current->fragments[i].frame = (current->fragments[i].frame + 1) & 3;
            }
        }
}
size_t tm1_effects(CollisionScene* s, s32 (*out)[5])
{
    size_t count = 0;
    for (int i = 0; i < 80; i++)
        if (s->effects[i][4] > 0)
            memcpy(out[count++], s->effects[i], 20);
    for (int i = 0; i < 10; i++)
        if (s->missiles[i].life > 0) {
            VECTOR3* v = &s->missiles[i].cs->pos;
            s32 e[5] = { v->vx, v->vy, v->vz, 3, 1 };
            memcpy(out[count++], e, 20);
        }
    for (int i = 0; i < 12; i++)
        if (s->drops[i].life > 0) {
            VECTOR3 v = s->drops[i].pos;
            s32 e[5] = { v.vx, v.vy, v.vz + 4, 6, 1 };
            memcpy(out[count++], e, 20);
        }
    for (int i = 0; i < 20; i++)
        if (s->fragments[i].life > 0) {
            VECTOR3 v = s->fragments[i].pos;
            s32 e[5] = { v.vx, v.vy, v.vz, 7, 1 };
            memcpy(out[count++], e, 20);
        }
    return count;
}
void do_simple_spark(VECTOR3* pos)
{
    arena_effect(pos, 1);
}
void do_puff(VECTOR3* pos)
{
    arena_effect(pos, 8);
}
void SetFXSheet(s32 n, s32 r, s32 g, s32 b)
{
    (void)n;
    (void)r;
    (void)g;
    (void)b;
}
void UAeffectActHitLights(Cs* cs, s32 n, u8* rgb)
{
    (void)cs;
    (void)n;
    (void)rgb;
}
void UAeffectSetFreezeLight(Cs* cs)
{
    (void)cs;
}
s16 MakeFakePotHole(void)
{
    numCheckPotHoles = numPotHoles + 1;
    return numPotHoles;
}
void UpdateNumPotHoles(void)
{
    if (numCheckPotHoles != numPotHoles) {
        current->pothole_fields += GetFieldsLastFrame();
        if (current->pothole_fields >= 101) {
            current->pothole_fields = 0;
            numCheckPotHoles = numPotHoles;
        }
    }
}
PotHole* GetPotHoleDat(s16 i)
{
    return &potHoles[i];
}

#include "tm1/HDP.H"
void tm1_world_point(CollisionScene* s, VECTOR3* point, s32* result)
{
    HdCsHit hit = { 0 };
    hit.tag0 = -1;
    hdpTraverse(&s->world, point, &hit);
    result[0] = hit.hit;
    result[1] = hit.depth;
    result[2] = hit.tag0;
    result[3] = hit.tag1;
}
// INTERACT.c fireFlameThrower and displayFlamethrowers, attached front/rear emitters.
int arena_launch_flame(Cs* cs, int forward)
{
    for (int i = 0; i < 5; i++) {
        if (current->flames[i].life)
            continue;
        current->flames[i].owner = cs;
        current->flames[i].forward = forward;
        current->flames[i].life = forward ? 25 : 12;
        current->flames[i].age = 0;
        if (forward)
            arena_effect(&cs->pos, 11);
        return 1;
    }
    return 0;
}

static void flame_burst(VECTOR3 pos)
{
    int frame = 1 + (arena_rand() & 3);
    for (int i = 0; i < 80; i++) {
        if (current->bursts[i].life)
            continue;
        current->bursts[i].pos = pos;
        current->bursts[i].life = 2;
        current->bursts[i].frame = frame;
        VECTOR3* vel = &current->bursts[i].vel;
        vel->vx = (arena_rand() & 7) - 3;
        vel->vy = (arena_rand() & 7) - 3;
        vel->vz = (arena_rand() & 15) > 5 ? 0 : (arena_rand() & 15) - 5;
        break;
    }
}

void arena_move_flames(void)
{
    for (int i = 0; i < 5; i++) {
        if (!current->flames[i].life)
            continue;
        Cs* cs = current->flames[i].owner;
        int forward = current->flames[i].forward;
        VECTOR3 offset = { 0, forward ? 32 : -25, 0 }, d;
        mathMulTransVec(&cs->mat, &offset, &d);
        VECTOR3 p = { cs->pos.vx + 4 * d.vx, cs->pos.vy + 4 * d.vy, cs->pos.vz + d.vz };
        int count = current->flames[i].age++;
        int limit = forward ? 20 : 12;
        current->flames[i].life--;
        int hit = 0;
        for (int j = 0; j < count && j < limit; j++) {
            p.vx += d.vx;
            p.vy += d.vy;
            p.vz += d.vz + 1;
            if (p.vz < 8 && cs->pos.vz > 0)
                p.vz = 7;
            flame_burst(p);
            if ((j & 2) && !hit) {
                hit = CAR_HD(&p, cs->unkC0, 17);
                if (hit > 0)
                    bulDispatchDamage(hit, 0, 0, 1, &d, 0);
            }
        }
    }
}

// EXPLODE.c animate_explosions runs before the particles are drawn.
void arena_animate_flames(void)
{
    for (int i = 0; i < 80; i++) {
        if (!current->bursts[i].life)
            continue;
        VECTOR3* p = &current->bursts[i].pos;
        VECTOR3 v = current->bursts[i].vel;
        p->vx += v.vx;
        p->vy += v.vy;
        p->vz += v.vz;
        current->bursts[i].life--;
        current->bursts[i].frame++;
    }
}

size_t tm1_flame_particles(CollisionScene* s, s32 (*out)[4])
{
    size_t n = 0;
    for (int i = 0; i < 80; i++) {
        if (!s->bursts[i].life)
            continue;
        VECTOR3 p = s->bursts[i].pos;
        s32 row[4] = { p.vx, p.vy, p.vz, s->bursts[i].frame };
        memcpy(out[n++], row, sizeof(row));
    }
    return n;
}
size_t tm1_contrail_faces(CollisionScene* s, s32 (*out)[14])
{
    size_t count = 0;
    for (int i = 0; i < 10; i++) {
        Missile* m = &s->missiles[i];
        if (!m->trail.visible)
            continue;
        for (int j = 0; j < 6; j++) {
            int h = (m->trail.head + 6 - j) % 6;
            int old = m->trail.ends[h][0], next = m->trail.ends[h][1];
            if (old == next)
                continue;
            for (int side = 0; side < 3; side++) {
                s32* row = out[count++];
                row[0] = (j * 8 + 3) / 6;
                row[1] = m->trail.color;
                for (int v = 0; v < 4; v++) {
                    SVECTOR local = m->trail.ring[v & 1 ? old : next][(side + (v >= 2)) % 3];
                    row[2 + 3 * v] = m->pos.vx + local.vx;
                    row[3 + 3 * v] = m->pos.vy + local.vy;
                    row[4 + 3 * v] = m->pos.vz + local.vz;
                }
            }
        }
    }
    return count;
}
size_t tm1_missile_poses(CollisionScene* s, s32 (*out)[13])
{
    size_t count = 0;
    for (int i = 0; i < 10; i++) {
        Missile* m = &s->missiles[i];
        if (m->life <= 0)
            continue;
        Cs* cs = m->cs;
        s32* row = out[count++];
        row[0] = m->model;
        row[1] = cs->pos.vx;
        row[2] = cs->pos.vy;
        row[3] = cs->pos.vz;
        for (int j = 0; j < 3; j++)
            for (int k = 0; k < 3; k++)
                row[4 + 3 * j + k] = cs->mat.m[j][k];
    }
    return count;
}
size_t tm1_taser_arcs(CollisionScene* s, VECTOR3 (*out)[16])
{
    size_t count = 0;
    for (size_t i = 0; i < 3; i++)
        if (s->tasers[i].visible)
            memcpy(out[count++], s->tasers[i].points, sizeof s->tasers[i].points);
    return count;
}

size_t tm1_bullet_segments(CollisionScene* s, VECTOR3 (*segments)[2])
{
    size_t count = 0;
    for (size_t i = 0; i < 40; i++) {
        Bullet* b = &s->bullets[i];
        if (b->life > 0) {
            segments[count][0] = b->prev;
            segments[count++][1] = b->pos;
        }
    }
    return count;
}
