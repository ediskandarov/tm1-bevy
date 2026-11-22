#include "tm1/arena_collision.h"
#include "tm1/arena_guns.h"
#include <stdbool.h>

// Native nodes contain host pointers. Every pointer slot in the JP data is
// resolved separately; only fixed-width scalar records are copied.
typedef struct DecodedNode {
    DbNode model;
    u32 address;
} DecodedNode;

typedef struct ArenaTarget {
    s32 type, instance, health;
    VECTOR3 pos;
} ArenaTarget;

typedef struct DamageNode {
    DbNode* node;
    u8 initial;
} DamageNode;

typedef struct StaticCop {
    DbSwitch* node;
    VECTOR3 pos;
    int phase, delay;
} StaticCop;

typedef struct BattleMusic {
    int hunt_delay, battle_delay;
    bool requested, playing;
} BattleMusic;

typedef struct CollisionScene {
    BattleMusic music, initial_music;
    int carry_notice_frames;
    ArenaMerc mercs[2];
    ArenaPed peds[6], initial_peds[6];
    int ped_count, peds_ready;
    StaticCop cops[25];
    int cop_cycle;
    ArenaTarget targets[50];
    DamageNode damage_nodes[2][30];
    int num_targets, num_damage_nodes[2];
    const u8* bytes;
    size_t size;
    u32 base;
    DbNode** nodes;
    u8* visiting;
    size_t slots;
    Cs world, taxi, opponents[8];
    Cs* first_car;
    int opponent_count, model_count;
    bool duel, beat_level;
    int wheel_distance[14];
    int64_t wheel_steps[14];
    struct { Cs* node; int count; } hit_lights[6];
    bool hit_light_flash;
    u8 displayed_lights[9];
    struct {
        int count;
        bool flip;
        u8 rgb[3], displayed[4];
    } hit_sheet;
    // Freeway barricades occupy 0..3. Rooftop crate and window pairs follow.
    DbSwitch* arena_switches[8];
    int drop_box_tick, window_tick;
    u8 initial_switches[8];
    DbSwitch* stands[10];
    VECTOR3 stand_positions[10];
    s32 radar_reports[18][4];
    size_t radar_count;
    u16 stand_slots, active_stands;
    int regen_timer;
    Bullet bullets[40];
    s32 drawn_bullets[40][8];
    size_t drawn_bullet_count;
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
    s32 drawn_fragments[20][5];
    size_t drawn_fragment_count;
    PotHole temporary_hole;
    int pothole_fields;
    bool has_temporary_hole;
    int battle_wait;
    DbTrans* pickups[50];
    u32 pickup_addr[50];
    int num_pickups, pickup_timer;
    Cs missile_cs[10];
    s32 effects[80][5];
    s32 primary_particles[80][5];
    size_t primary_particle_count;
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
    DecodedNode* record = calloc(1, sizeof(*record));
    if (!record)
        return NULL;
    record->address = addr;
    DbNode* n = &record->model;
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
        if (!load(s, addr, 4, &n->rot.mat, 36))
            return NULL;
        if (!children(s, addr, 40, n->rot.child, n->rot.nChild, depth))
            return NULL;
        break;
    case DB_OP_TRANS:
        if (!load(s, addr, 4, &n->trans.ofs, 16))
            return NULL;
        if (!children(s, addr, 20, n->trans.child, n->trans.nChild, depth))
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
        int slot = -1;
        if (n->h.kind == UA_BARRICADE && n->sw.sub >= 1 && n->sw.sub <= 4)
            slot = n->sw.sub - 1;
        else if (n->h.kind == UA_DROP_BOX || n->h.kind == UA_FALLING_BOX)
            slot = n->h.kind == UA_DROP_BOX ? 4 : 5;
        else if (n->h.kind == UA_UNK_1001) {
            slot = 6;
        } else if (n->h.kind == UA_BREAKING_WINDOW)
            slot = 7;
        if (slot >= 0) {
            s->arena_switches[slot] = &n->sw;
            s->initial_switches[slot] = n->sw.sel;
        }
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
// DB.c visits the last child first, including inactive switches and both
// visibility branches. Target IDs and the destructible node indices are independent.
static int target_health(int type)
{
    switch (type) {
    case 600: case 1017: return 8;
    case 601: case 716: case 720: return 3;
    case 610: return 5;
    case 611: return 15;
    case 612: return 10;
    case 613: return 20;
    case 715: return 12;
    case 902: return 25;
    case 1000: return 100;
    case 1002: case 1009: case 1010: return 4;
    case 1015: return 2;
    default: return 1;
    }
}

// Zero selects the UA_SW switch table, one the child-count table.
static int damage_node_class(int type)
{
    switch (type) {
    case 715: case 716: case 720: case 721: case 730:
    case 1003: case 1004: case 1005: case 1008: case 1015:
        return 1;
    case 600: case 601: case 610: case 611: case 612: case 613: case 770: case 902:
    case 930: case 931: case 1000: case 1001: case 1002: case 1009: case 1010: case 1017:
        return 0;
    default:
        return -1;
    }
}

static bool scan_targets(CollisionScene* s, DbNode* n, VECTOR3 pos, unsigned* visits)
{
    if (!n)
        return true;
    if (++*visits > 1000000)
        return false;
    DbNode** children = NULL;
    int count = 0;
    int op = n->h.op;
    int type = n->h.kind;
    switch (op) {
    case DB_OP_GROUP:
        children = n->grp.child;
        count = n->grp.nChild;
        break;
    case DB_OP_ROT: {
        VECTOR3 sum = {
            pos.vx + n->rot.mat.t[0],
            pos.vy + n->rot.mat.t[1],
            pos.vz + n->rot.mat.t[2],
        };
        mathMulVec(&n->rot.mat, &sum, &pos);
        children = n->rot.child;
        count = n->rot.nChild;
        break;
    }
    case DB_OP_TRANS:
        pos.vx += n->trans.ofs.vx;
        pos.vy += n->trans.ofs.vy;
        pos.vz += n->trans.ofs.vz;
        children = n->trans.child;
        count = n->trans.nChild;
        break;
    case DB_OP_SWITCH:
        children = n->sw.child;
        count = n->sw.nChild;
        break;
    case DB_OP_ANIM:
        children = n->anim.child;
        count = n->anim.nChild;
        break;
    case DB_OP_LOD:
        for (int i = n->lod.nRec - 1; i >= 0; i--)
            if (!scan_targets(s, n->lod.rec[i]->child, pos, visits))
                return false;
        break;
    case DB_OP_CTRL:
        return scan_targets(s, n->ctrl.childIn, pos, visits)
            && scan_targets(s, n->ctrl.childOut, pos, visits);
    }

    if (op == DB_OP_SWITCH && type == 920 && n->sw.sub >= 1 && n->sw.sub <= 10)
        s->stand_positions[n->sw.sub - 1] = pos;
    if (op == DB_OP_SWITCH && type >= 753 && type <= 755 && n->sw.sub >= 1 && n->sw.sub <= 25)
        s->cops[n->sw.sub - 1] = (StaticCop){ &n->sw, pos, 0, 6 };
    int group = damage_node_class(type);
    if (group >= 0 && op != DB_OP_BOX && op != DB_OP_VOLUME && op != DB_OP_UNK_6 && op != DB_OP_UNK_10) {
        u16 sub;
        if (!load(s, ((DecodedNode*)n)->address, group ? 22 : 4, &sub, sizeof(sub)))
            return false;
        int index = s->num_damage_nodes[group];
        if (op == (group ? DB_OP_GROUP : DB_OP_SWITCH) && type != 930 && type != 931 && index < 30) {
            s->damage_nodes[group][index] = (DamageNode){ n, group ? n->grp.nChild : n->sw.sel };
            sub = index;
            if (group)
                n->grp.sub = sub;
            else
                n->sw.sub = sub;
            s->num_damage_nodes[group]++;
        }
        if (s->num_targets < 50)
            s->targets[s->num_targets++] = (ArenaTarget){ type, sub, target_health(type), pos };
    }
    for (int i = count - 1; i >= 0; i--)
        if (!scan_targets(s, children[i], pos, visits))
            return false;
    return true;
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
static int pedestrian_kind(int kind)
{
    switch (kind) {
    case 751: case 756: case 760: case 772: case 773: case 775:
    case 776: case 777: case 778: case 781: case 782: return 1;
    default: return 0;
    }
}

static bool register_pedestrians(CollisionScene* s, u32 root)
{
    u8 count;
    if (!load(s, root, 20, &count, 1)) return false;
    // The retail databases place each actor group inside an untagged root.
    // dbScanForInteractiveStuff visits the last child first.
    for (int i = count - 1; i >= 0 && s->ped_count < 6; i--) {
        u32 addr;
        DbNodeHdr hdr;
        if (!load(s, root, 24 + 4 * i, &addr, 4) || !load(s, addr, 0, &hdr, 4)) return false;
        if (hdr.op != DB_OP_GROUP || !pedestrian_kind(hdr.kind)) continue;
        DbNode* model = decode(s, addr, 0);
        if (!model || !model->grp.nChild) return false;
        DbNode* sw = model->grp.child[0];
        if (sw->h.op == DB_OP_TRANS && sw->trans.nChild) sw = sw->trans.child[0];
        if (sw->h.op != DB_OP_SWITCH) return false;
        ArenaPed* p = &s->peds[s->ped_count];
        p->model.epNode = model;
        p->model.unkC0 = 150 + s->ped_count++;
        p->model.next = s->first_car;
        s->first_car = &p->model;
        p->id = hdr.kind;
        p->node = &sw->sw;
        switch (p->id) {
        case 756: case 778: p->walk_end = 3; p->alert_end = 4; p->hit_end = 5; break;
        case 751: p->walk_end = 1; p->alert_end = 5; p->hit_end = 7; break;
        case 760: p->walk_end = p->alert_end = p->hit_end = 5; break;
        case 773: p->walk_end = p->alert_end = 5; p->hit_end = 6; break;
        default: p->walk_end = 2; p->alert_end = 4; p->hit_end = 6; break;
        }
    }
    return true;
}

u32 tm1_ped_model(const CollisionScene* s, int slot)
{
    if (slot < 0 || slot >= s->ped_count) return 0;
    return ((DecodedNode*)s->peds[slot].model.epNode)->address;
}
void tm1_start_ped(CollisionScene* s, int slot, int level, int route, int end)
{
    ArenaPed* p = &s->peds[slot];
    arena_choose_ped_path(p, level, route, end);
    // Every initial path heading is a quarter turn. Later impacts may set an arbitrary yaw.
    const int sin_q[4] = {0, 4096, 0, -4096};
    int q = (p->model.rot.vz >> 10) & 3;
    int sn = sin_q[q], cs = sin_q[(q + 1) & 3];
    p->model.mat.m[0][0] = p->model.mat.m[1][1] = cs;
    p->model.mat.m[0][1] = -sn;
    p->model.mat.m[1][0] = sn;
    p->model.mat.m[2][2] = 4096;
    p->model.wmat = p->model.mat;
    p->model.unk0C = 1;
    s->initial_peds[slot] = *p;
    s->peds_ready = slot + 1;
}
void tm1_move_pedestrians(int level, const VECTOR3* pos)
{
    for (int i = 0; i < current->peds_ready; i++)
        arena_move_ped(&current->peds[i], level, pos);
}
int tm1_ped_pose(const CollisionScene* s, int slot, int* pose)
{
    if (slot >= s->peds_ready) return 0;
    const ArenaPed* p = &s->peds[slot];
    pose[0] = p->node->sel;
    pose[1] = p->model.pos.vx;
    pose[2] = p->model.pos.vy;
    pose[3] = p->model.pos.vz;
    for (int j = 0; j < 9; j++)
        pose[4 + j] = p->model.mat.m[j / 3][j % 3];
    pose[13] = p->model.drawMode;
    return 1;
}

static void reset_merc(ArenaMerc* merc, int slot)
{
    merc->timer = 200 * (slot + 1);
    merc->hover_sound = 0;
    merc->phase = merc->shots = merc->altitude = 0;
    merc->site = -1;
    merc->node->sel = merc->initial_branch;
    merc->model.pos = (VECTOR3){0};
}
void tm1_move_mercs(int level, const VECTOR3* player)
{
    arena_move_mercs(current->mercs, level, player);
}
int tm1_merc_pose(const CollisionScene* scene, int slot, int* out)
{
    const ArenaMerc* merc = &scene->mercs[slot];
    if (!merc->node)
        return 0;
    out[0] = merc->node->sel;
    out[1] = merc->model.pos.vx;
    out[2] = merc->model.pos.vy;
    out[3] = merc->model.pos.vz;
    out[4] = merc->hover_sound;
    return 1;
}
CollisionScene* tm1_scene_load(
    const u8* data, size_t size, u32 base, u32 world, u32 taxi, const u32* opponents, size_t count, int duel)
{
    if (!count || count > 8 || (duel && count != 1))
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
    s->duel = duel;
    s->model_count = count;
    s->opponent_count = duel ? 0 : count;
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
        DbNodeHdr hdr;
        if (!load(s, root, 0, &hdr, 4))
            goto invalid;
        if (hdr.op == DB_OP_GROUP && hdr.kind == 0 && !register_pedestrians(s, root))
            goto invalid;
        if (hdr.op == DB_OP_GROUP && (hdr.kind == UA_BAZ_COP || hdr.kind == UA_HOVER_COP)) {
            int slot = hdr.kind == UA_HOVER_COP;
            ArenaMerc* merc = &s->mercs[slot];
            DbNode* model = decode(s, root, 0);
            if (!model || !model->grp.nChild)
                goto invalid;
            DbNode* sw = model->grp.child[0];
            if (sw->h.op == DB_OP_TRANS && sw->trans.nChild)
                sw = sw->trans.child[0];
            if (sw->h.op != DB_OP_SWITCH)
                goto invalid;
            merc->node = &sw->sw;
            merc->initial_branch = sw->sw.sel;
            reset_merc(merc, slot);
            Cs* cs = &merc->model;
            cs->epNode = model;
            cs->drawMode = cs->unk0C = 1;
            cs->unkC0 = 100 + slot;
            for (int axis = 0; axis < 3; axis++)
                cs->mat.m[axis][axis] = cs->wmat.m[axis][axis] = 4096;
            cs->next = s->first_car;
            s->first_car = cs;
        }
        for (size_t j = 0; j <= count; j++) {
            if (root != (j ? opponents[j - 1] : taxi))
                continue;
            Cs* cs = j ? &s->opponents[j - 1] : &s->taxi;
            if (cs->drawMode || !(cs->epNode = decode(s, root, 0)))
                goto invalid;
            cs->unkC0 = j ? (duel ? 2 : 49 + j) : 1;
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
    unsigned visits = 0;
    if (!scan_targets(s, s->world.epNode, (VECTOR3){0}, &visits))
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

// PlayGame records each target before the next actor can move or damage it.
void tm1_begin_radar_reports(void) { current->radar_count = 0; }
void tm1_report_radar(const PlayerCar* player, const VECTOR3* pos, int slot)
{
    s32* report = current->radar_reports[current->radar_count++];
    report[0] = slot;
    report[1] = pos->vx - player->motion.pos.vx;
    report[2] = pos->vy - player->motion.pos.vy;
    report[3] = player->motion.rot.vz;
}
void tm1_report_radar_stands(const PlayerCar* player)
{
    for (int slot = 0; slot < 10; slot++)
        if (current->active_stands & (1u << slot))
            tm1_report_radar(player, &current->stand_positions[slot], -1);
}
size_t tm1_radar_reports(const CollisionScene* scene, s32 reports[18][4])
{
    memcpy(reports, scene->radar_reports, scene->radar_count * sizeof(*reports));
    return scene->radar_count;
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
        static const int health_delay[] = { 2000, 4000, 8000 };
        current->regen_timer = health_delay[uaGetDifficulty()];
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

void tm1_new_life(CollisionScene* s)
{
    s->carry_notice_frames = 0;
    s->music.playing = false;
    memset(s->drops, 0, sizeof s->drops);
    memset(s->flames, 0, sizeof s->flames);
    memset(s->tasers, 0, sizeof s->tasers);
}
void tm1_reset_projectiles(CollisionScene* s)
{
    memset(s->bullets, 0, sizeof s->bullets);
    s->drawn_bullet_count = 0;
    memset(s->effects, 0, sizeof s->effects);
    s->primary_particle_count = 0;
    s->has_temporary_hole = false;
    s->pothole_fields = 0;
    memset(&s->temporary_hole, 0, sizeof(PotHole));
    memset(s->fragments, 0, sizeof s->fragments);
    s->drawn_fragment_count = 0;
    tm1_new_life(s);
    s->music = s->initial_music;
    memset(s->bursts, 0, sizeof s->bursts);
    s->battle_wait = 0;
    s->beat_level = false;
    memset(s->wheel_distance, 0, sizeof s->wheel_distance);
    memset(s->wheel_steps, 0, sizeof s->wheel_steps);
    memset(s->hit_lights, 0, sizeof s->hit_lights);
    memset(s->displayed_lights, 0, sizeof s->displayed_lights);
    s->hit_light_flash = false;
    s->taxi.env = NULL;
    for (int i = 0; i < s->model_count; i++)
        s->opponents[i].env = NULL;
    memset(&s->hit_sheet, 0, sizeof s->hit_sheet);
    for (int i = 0; i < s->num_targets; i++)
        s->targets[i].health = target_health(s->targets[i].type);
    for (int group = 0; group <= 1; group++) {
        for (int i = 0; i < s->num_damage_nodes[group]; i++) {
            DamageNode model = s->damage_nodes[group][i];
            if (group)
                model.node->grp.nChild = model.initial;
            else
                model.node->sw.sel = model.initial;
        }
    }
    for (int i = 0; i < 2; i++)
        if (s->mercs[i].node)
            reset_merc(&s->mercs[i], i);
    for (int i = 0; i < 25; i++) {
        StaticCop* cop = &s->cops[i];
        if (cop->node) {
            cop->phase = 0;
            cop->delay = 6;
            cop->node->sel = 0;
        }
    }
    for (int i = 0; i < s->peds_ready; i++) {
        s->peds[i] = s->initial_peds[i];
        s->peds[i].node->sel = 1;
    }
    s->drop_box_tick = s->window_tick = 0;
    for (int i = 0; i < 8; i++)
        if (s->arena_switches[i])
            s->arena_switches[i]->sel = s->initial_switches[i];
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
void tm1_active_opponents(CollisionScene* scene, int count)
{
    if (scene->opponent_count == count)
        return;
    for (int i = 0; i < scene->model_count; i++)
        scene->opponents[i].drawMode = i < count;
    scene->opponent_count = count;
    scene->beat_level = false; /* uaInitBossCar reopens the Rooftops battle. */
}

bool tm1_level_won(const CollisionScene* scene)
{
    return scene->beat_level;
}
u8 uaHasPlayerBeatThisLevel(void)
{
    return current && current->beat_level;
}
void tm1_check_victory(const PlayerCar* player, const AICar* cars, int count)
{
    if (current->beat_level || player->flags[0])
        return;
    for (int i = 0; i < count; i++)
        if (cars[i].stats.unk40 > 0)
            return;
    current->beat_level = true;
}

s16 GetNumPlayers(void)
{
    return current && current->duel ? 2 : 1;
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
    return i == 1 && current->duel ? &current->opponents[0] : &current->taxi;
}
Cs* GetAICs3D(s16 i)
{
    return &current->opponents[i];
}
u8 uaIsCarCS(Cs* cs, u8* player, s16* idx)
{
    if (cs == &current->taxi || (current->duel && cs == &current->opponents[0])) {
        *idx = cs != &current->taxi;
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
    if (id == 1 || (current->duel && id == 2) || (id >= 50 && id < 50 + current->opponent_count)) {
        *player = id < 50;
        *idx = *player ? id - 1 : id - 50;
        return 1;
    }
    return 0;
}
int tm1_target_record(const CollisionScene* s, int index, ArenaTarget* out)
{
    if (index < 0 || index >= s->num_targets)
        return 0;
    *out = s->targets[index];
    return 1;
}
int arena_hit_target(const VECTOR3* p, int owner, int radius)
{
    for (int i = 0; i < current->num_targets; i++) {
        ArenaTarget* t = &current->targets[i];
        if (t->health > 0 && t->type != -owner
            && (abs(p->vx - t->pos.vx) >> 3) < radius + 5
            && (abs(p->vy - t->pos.vy) >> 3) < radius + 5
            && (abs(p->vz - t->pos.vz) >> 3) < radius + 5)
            return 500 + i;
    }
    for (int i = 0; i < current->peds_ready; i++) {
        ArenaPed* ped = &current->peds[i];
        VECTOR3 pos = ped->model.pos;
        if (ped->id != 760 && abs(p->vx - pos.vx) < 56 && abs(p->vy - pos.vy) < 56
            && abs(p->vz - pos.vz) < 56) {
            arena_ped_hit(ped, GetPlayerInfo(0)->motion.rot.vz);
            return 8;
        }
    }
    for (int i = 0; i < 2; i++) {
        ArenaMerc* merc = &current->mercs[i];
        VECTOR3* pos = &merc->model.pos;
        if (merc->node && (abs(p->vx - pos->vx) >> 3) < radius + 4
            && (abs(p->vy - pos->vy) >> 3) < radius + 4
            && (abs(p->vz - pos->vz) >> 3) < radius + 4)
            return 100 + i;
    }
    return 0;
}

static void damage_target(int type, int sub, int damage)
{
    int group = damage_node_class(type);
    if (group < 0 || type == 930 || type == 931)
        return;
    // target_takehit keeps the last living match when a model is instanced.
    for (int i = current->num_targets - 1; i >= 0; i--) {
        ArenaTarget* t = &current->targets[i];
        if (t->type != type || t->instance != sub || t->health <= 0)
            continue;
        t->health -= damage;
        if (t->health > 0)
            return;
        if ((unsigned)sub < (unsigned)current->num_damage_nodes[group]) {
            DbNode* model = current->damage_nodes[group][sub].node;
            if (group)
                model->grp.nChild = 0;
            else if (model->sw.nChild > 1)
                model->sw.sel = 1;
        }
        VECTOR3 pos = t->pos;
        do_smoke(&pos);
        do_puff(&pos);
        switch (type) {
        case 1001:
            current->window_tick = 1;
            arena_effect(&pos, 31);
            arena_effect(&pos, 33);
            break;
        case 600: case 601: case 610: case 612: case 1017:
            arena_effect(&pos, 28);
            arena_effect(&pos, 35);
            break;
        case 611: case 613: {
            VECTOR3 offset = {pos.vx - 40, pos.vy + 40, pos.vz + 30};
            arena_effect(&offset, 29);
            arena_effect(&offset, 28);
            do_smoke(&offset);
            do_puff(&offset);
            do_smoke(&pos);
            break;
        }
        case 720: case 770:
            arena_effect(&pos, 36);
            break;
        case 730: {
            arena_effect(&pos, 37);
            arena_effect(&pos, 35);
            int slot = FindFreeDropWeap();
            if (slot >= 0) {
                DropWeap* oil = &current->drops[slot];
                oil->idx = 0;
                oil->kind = WEAPON_OIL;
                oil->type = HIT_OIL;
                oil->life = 400;
                oil->pos = pos;
                oil->power = 0;
            }
            break;
        }
        case 1000:
            for (int j = 0; j < 45; j++) {
                VECTOR3 cloud;
                cloud.vx = arena_rand() % 2400 + 9600;
                cloud.vy = arena_rand() % 2400 + 4000;
                cloud.vz = (arena_rand() & 1023) + 8000;
                arena_effect(&cloud, 34);
                if (arena_rand() & 1)
                    arena_effect(&cloud, 30);
            }
            break;
        }
        return;
    }
}

int tm1_damage_branch(const CollisionScene* s, u32 addr)
{
    if (addr < s->base || (addr & 3) || (addr - s->base) / 4 >= s->slots)
        return 0;
    DbNode* n = s->nodes[(addr - s->base) / 4];
    if (!n)
        return 0;
    if (n->h.op == DB_OP_GROUP)
        return n->grp.nChild == 0;
    return n->h.op == DB_OP_SWITCH ? n->sw.sel : 0;
}

s32 bulDispatchDamage(s32 id, s32 tag, s32 sub, s32 dmg, VECTOR3* dir, s32 owner)
{
    if (id >= 500 && id < 500 + current->num_targets) {
        ArenaTarget* t = &current->targets[id - 500];
        tag = t->type;
        sub = t->instance;
        id = 8;
    }
    if ((id >= 150 && id < 150 + current->peds_ready) || ((id == 8 || id <= 0)
        && (tag == 760 || tag == 777 || tag == 778 || tag == 781 || tag == 782))) {
        int kind = id >= 150 ? current->peds[id - 150].id : tag;
        for (int i = current->peds_ready - 1; i >= 0; i--) {
            if (current->peds[i].id == kind) {
                arena_ped_hit(&current->peds[i], GetPlayerInfo(0)->motion.rot.vz);
                break;
            }
        }
        return 1;
    }
    if (id == 100 || id == 101 || ((id <= 0 || id == 8) && (tag == UA_BAZ_COP || tag == UA_HOVER_COP))) {
        int slot = id == 101 || ((id <= 0 || id == 8) && tag == UA_HOVER_COP);
        ArenaMerc* merc = &current->mercs[slot];
        if (merc->node) {
            VECTOR3* pos = &merc->model.pos;
            arena_effect(pos, 36);
            merc->node->sel = 4;
            do_smoke(pos);
            arena_effect(pos, 44 + arena_rand() % 7);
            arena_effect(pos, 31);
            arena_effect(pos, 28);
            pos->vz = 0;
            arena_effect(pos, 35);
            merc->phase = 0;
            merc->timer = 500 + arena_rand() % 250;
            pos->vz = 1000;
            merc->site = -1;
        }
        return 1;
    }
    if ((id <= 0 || id == 8) && tag >= 753 && tag <= 755 && sub >= 1 && sub <= 25) {
        StaticCop* cop = &current->cops[sub - 1];
        arena_effect(&cop->pos, 36);
        if (cop->node && cop->phase < 4) {
            cop->phase = 4;
            cop->node->sel = 4;
        }
        return 1;
    }
    if ((id <= 0 || id == 8) && tag == UA_DROP_BOX) {
        DbSwitch* release = current->arena_switches[4];
        if (release && release->nChild > 1)
            release->sel = 1;
        current->drop_box_tick = 1;
        return 1;
    }
    if (id <= 0 || id == 8) {
        if (damage_node_class(tag) >= 0 && tag != 930 && tag != 931) {
            damage_target(tag, sub, dmg);
            return 1;
        }
        return tag == 603 || tag == 702 || tag == 703 || tag == 705 || tag == 707
            || tag == 708 || tag == 713;
    }
    (void)owner;
    s8 player;
    u16 slot;
    if (uaIsCarMatID(id, &player, &slot))
        carTakeHit(id, dmg, dir, 0);
    return 1;
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

    arena_crush_sound(isPlayerA ? PLAYER_CAR(carA)->playerIdx : 1 + AI_CAR(carA)->playerIdx);
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
            case 1:
                arena_effect(&pos, 34);
                break;
            case 3:
                arena_effect(&pos, 31);
                break;
            case 0:
                do_smoke(&pos);
                break;
            case 2:
                arena_effect(&pos, 28);
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
    if (GetNumPlayers() == 2) {
        int slot = 1 - car->playerIdx;
        PlayerCar* rival = GetPlayerInfo(slot);
        if (rival->uaIndex == car->stats.unk30) {
            check_smash(&car->stats, &car->motion, car->tires, car->flags,
                GetPlayerCs3D(slot), &rival->stats, rival->flags);
            return;
        }
    }
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
    static const int weapon_delay[] = { 150, 250, 350 };
    if (current->pickup_timer < weapon_delay[uaGetDifficulty()]) {
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
    if (cap <= 0 && tag != 430 && player) {
        current->carry_notice_frames = 30;
        return;
    }
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
    if (slot >= 0 || tag == 430) {
        CarMotion* motion = player ? &PLAYER_CAR(c)->motion : &AI_CAR(c)->motion;
        arena_effect(&motion->pos, tag); /* UA pickup kind identifies its sound voice. */
    }
    if (sub > 0 && sub <= 50 && current->pickups[sub - 1])
        current->pickups[sub - 1]->nChild = 0;
    pad->totalAmmo = 0;
    for (int i = 0; i < WEAPON_REGULAR_MAX; i++)
        pad->totalAmmo += pad->ammo[i];
    if (player && tag != 430 && (s16)pad->totalAmmo == (s16)pad->maxAmmo)
        current->carry_notice_frames = 30;
}

int tm1_carry_notice_frames(const CollisionScene* scene)
{
    return scene->carry_notice_frames;
}
void UAeffectBarricade(s32 tag, s32 sub, s32 bearing)
{
    if (!current || tag != UA_BARRICADE || sub < 1 || sub > 4 || (u32)bearing > 1)
        return;
    DbSwitch* node = current->arena_switches[sub - 1];
    if (node && bearing + 1 < node->nChild)
        node->sel = bearing + 1;
}

static void advance_rooftop_switch(DbSwitch* node, int* tick)
{
    if (!*tick)
        return;
    int branch = *tick - 1;
    if (node && branch < node->nChild)
        node->sel = branch;
    if (++*tick == 10) {
        if (node && node->nChild > 9)
            node->sel = 9;
        *tick = 0;
    }
}

static void static_cop_fire(StaticCop* cop, const VECTOR3* target)
{
    VECTOR3 pos = cop->pos;
    int type = cop->node->h.kind;
    cop->phase = 3;
    cop->delay = 4;
    if (type == 753) {
        pos.vz += 40;
        VECTOR3 aim;
        aim.vx = target->vx - pos.vx - 64 + (arena_rand() & 127);
        aim.vy = target->vy - pos.vy - 64 + (arena_rand() & 127);
        aim.vz = target->vz - pos.vz;
        create_bullet(-type, &aim, &pos, 1);
        arena_flash(&pos);
    } else {
        pos.vz += 52;
        int dx = target->vx - pos.vx + arena_rand() % 200 - 100;
        int yaw = ratan2(dx, target->vy - pos.vy) + arena_rand() % 200 - 100;
        arena_cop_missile(-type, yaw, &pos);
        arena_flash(&pos);
        do_puff(&pos);
        cop->delay = 7 + (arena_rand() & 7);
    }
}

void arena_request_battle_music(void)
{
    if (current)
        current->music.requested = true;
}

// UA.c PlayGame only calls the battle routine when an opponent requested it.
// UA_SOUND.c counts calls to each routine, independently of the field interval.
void tm1_update_battle_music(CollisionScene* scene, int level, int engaged)
{
    if (level != LEVEL2_WAREHOUSE && level != LEVEL4_CITY_PARK && level != LEVEL5_CYBURBIA)
        return;
    BattleMusic* music = &scene->music;
    if (engaged && !music->requested)
        return;
    if (engaged)
        music->requested = false;
    int* delay = engaged ? &music->battle_delay : &music->hunt_delay;
    int* other = engaged ? &music->hunt_delay : &music->battle_delay;
    if (*other > 0) {
        (*other)--;
    } else {
        if (--*delay <= 0)
            *delay = 30;
        music->playing = engaged != 0;
    }
}

int tm1_battle_music_playing(const CollisionScene* scene)
{
    return scene->music.playing;
}

void tm1_continue_stage_state(CollisionScene* scene, const CollisionScene* previous)
{
    scene->cop_cycle = previous->cop_cycle;
    scene->music = previous->music;
    scene->music.playing = false;
    scene->initial_music = scene->music;
}

void tm1_move_static_cops(const VECTOR3* player)
{
    for (int i = current->cop_cycle; i < 25; i += 6) {
        StaticCop* cop = &current->cops[i];
        if (!cop->node)
            continue;
        int dx = abs(player->vx - cop->pos.vx);
        int dy = abs(player->vy - cop->pos.vy);
        int distance = dx >= dy ? dx + (dy >> 1) : dy + (dx >> 1);
        switch (cop->phase) {
        case 0:
            if (distance < 1200)
                cop->phase = 1;
            break;
        case 1:
            cop->phase = distance < 1200 ? 2 : 0;
            if (cop->phase == 2)
                cop->delay = 2;
            break;
        case 2:
            if (--cop->delay <= 0)
                static_cop_fire(cop, player);
            break;
        case 3:
            if (--cop->delay < 3)
                cop->phase = distance < 1200 ? 2 : 1;
            break;
        case 4:
            cop->phase = 5;
            break;
        case 5:
            cop->phase = 6;
            cop->delay = 900;
            break;
        case 6:
            if (--cop->delay <= 0)
                cop->phase = 0;
            break;
        }
        if (cop->phase < cop->node->nChild)
            cop->node->sel = cop->phase;
    }
    current->cop_cycle = (current->cop_cycle + 1) % 6;
}

void arena_flash(VECTOR3* pos)
{
    arena_effect(pos, 38 + arena_rand() % 6);
}

void tm1_move_rooftop_targets(void)
{
    advance_rooftop_switch(current->arena_switches[5], &current->drop_box_tick);
    advance_rooftop_switch(current->arena_switches[7], &current->window_tick);
}

uint64_t tm1_arena_switch_states(const CollisionScene* scene)
{
    uint64_t states = 0;
    for (int i = 0; i < 8; i++)
        if (scene->arena_switches[i])
            states |= (uint64_t)scene->arena_switches[i]->sel << (8 * i);
    return states;
}
// The first rival sample belongs to player two in a split-screen bank.
void arena_special_effect(VECTOR3* pos, s32 owner)
{
    s32 effect = owner >= 50 ? 18 + owner - 50 : (owner == 2 ? 18 : 10);
    arena_record_effect(pos, effect);
}

// EXPLODE.c find_free_explosion scans the shared pyro array in slot order.
static int free_explosion(void)
{
    for (int i = 0; i < 80; i++)
        if (!current->effects[i][4] && !current->bursts[i].life)
            return i;
    return -1;
}

static int effect_lifetime(int kind)
{
    switch (kind) {
    case 1: return 4;
    case 55: case 56: case 28: case 37: return 32;
    case 8: case 34: return 88;
    case 44: case 45: case 46: case 47: case 48: case 49: case 50:
    case 53: return 2;
    case 38: case 39: case 40: case 41: case 42: case 43: return 3;
    case 35: return 104;
    case 31: return 24;
    case 32: case 29: case 30: return 16;
    default: return 0; // Sound-only requests have no pyro entry.
    }
}

void arena_effect(VECTOR3* pos, s32 kind)
{
    if (!current)
        return;
    arena_record_effect(pos, kind);
    int life = effect_lifetime(kind);
    if (!life)
        return;
    int slot = free_explosion();
    if (slot < 0)
        return;
    s32 row[] = {pos->vx, pos->vy, pos->vz, kind, life};
    memcpy(current->effects[slot], row, sizeof(row));
}

void tm1_tick_carry_notice(void)
{
    if (current->carry_notice_frames > 0)
        current->carry_notice_frames--;
}
static size_t copy_particles(const CollisionScene* scene, int secondary, s32 (*out)[5])
{
    size_t n = 0;
    for (int i = 0; i < 80; i++) {
        const s32* e = scene->effects[i];
        if (scene->bursts[i].life) {
            if (secondary && scene->bursts[i].life == 1)
                continue;
            VECTOR3 p = scene->bursts[i].pos;
            int age = 2 - scene->bursts[i].life;
            int first = scene->bursts[i].frame - age;
            s32 row[] = {p.vx, p.vy, p.vz, 56 + first, age};
            memcpy(out[n++], row, sizeof(row));
        } else if (e[4]) {
            // draw_explosions omits age one of the final animation loop in view two.
            if (secondary) {
                int final_loop = effect_lifetime(e[3]);
                if (e[3] == 35)
                    final_loop = 8;
                else if (e[3] == 8 || e[3] == 34)
                    final_loop = 32;
                if (e[4] == final_loop - 1)
                    continue;
            }
            memcpy(out[n++], e, sizeof(*out));
        }
    }
    return n;
}

void arena_animate_effects(void)
{
    for (int i = 0; i < 80; i++) {
        s32* e = current->effects[i];
        if (e[4] == 0)
            continue;
        if (e[3] == 8 || e[3] == 34)
            e[2] += 10;
        else if (e[3] == 32)
            e[2] += 6;
        e[4]--;
    }
    current->primary_particle_count = copy_particles(current, 0, current->primary_particles);
}

size_t tm1_drawn_particles(const CollisionScene* scene, int secondary, s32 (*out)[5])
{
    if (secondary)
        return copy_particles(scene, 1, out);
    memcpy(out, scene->primary_particles, scene->primary_particle_count * sizeof(*out));
    return scene->primary_particle_count;
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
static size_t copy_fragment_poses(const CollisionScene* scene, s32 (*out)[5])
{
    size_t n = 0;
    for (int i = 0; i < 20; i++) {
        if (scene->fragments[i].life > 0) {
            VECTOR3 pos = scene->fragments[i].pos;
            s32 pose[] = {pos.vx, pos.vy, pos.vz, scene->fragments[i].variant & 1, scene->fragments[i].frame};
            memcpy(out[n++], pose, sizeof(pose));
        }
    }
    return n;
}

void arena_capture_fragments(void)
{
    current->drawn_fragment_count = copy_fragment_poses(current, current->drawn_fragments);
}

size_t tm1_fragment_poses(const CollisionScene* scene, int second_view, s32 (*out)[5])
{
    if (second_view)
        return copy_fragment_poses(scene, out);
    memcpy(out, scene->drawn_fragments, scene->drawn_fragment_count * sizeof(*out));
    return scene->drawn_fragment_count;
}

size_t tm1_drawn_drops(CollisionScene* s, int view, s32 (*out)[4])
{
    size_t n = 0;
    for (int i = 0; i < 12; i++) {
        DropWeap* d = &s->drops[i];
        if (d->drawn_kind[view]) {
            s32 pose[4] = { d->drawn_pos[view].vx, d->drawn_pos[view].vy, d->drawn_pos[view].vz, d->drawn_kind[view] };
            memcpy(out[n++], pose, sizeof(pose));
        }
    }
    return n;
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
    arena_effect(pos, 32);
}
void SetFXSheet(s32 n, s32 r, s32 g, s32 b)
{
    if (!current)
        return;
    current->hit_sheet.count = n;
    current->hit_sheet.rgb[0] = r;
    current->hit_sheet.rgb[1] = g;
    current->hit_sheet.rgb[2] = b;
}

// DisplayFXSheet links one of two primitives and fades both before GPU submission.
void tm1_display_hit_sheet(void)
{
    CollisionScene* scene = current;
    bool flip = scene->hit_sheet.flip;
    scene->hit_sheet.flip = !flip;
    if (scene->hit_sheet.count <= 0) {
        scene->hit_sheet.displayed[3] = 0;
        return;
    }
    if (--scene->hit_sheet.count < 8)
        for (int i = 0; i < 3; i++)
            scene->hit_sheet.rgb[i] = (scene->hit_sheet.rgb[i] * 3 + 128) >> 2;
    memcpy(scene->hit_sheet.displayed, scene->hit_sheet.rgb, 3);
    scene->hit_sheet.displayed[3] = flip ? 255 : 128;
}
void tm1_hit_sheet(const CollisionScene* scene, u8* rgba)
{
    memcpy(rgba, scene->hit_sheet.displayed, 4);
}
// Light environments are identities on the host. The renderer supplies their colors.
struct LightEnv { u8 index; };
static LightEnv red_flash = {1}, freeze_light = {2};

void UAeffectActHitLights(Cs* cs, s32 n, u8* rgb)
{
    (void)rgb;
    if (!current)
        return;
    for (int i = 0; i < 6; i++) {
        if (current->hit_lights[i].node == cs) {
            current->hit_lights[i].count = n;
            return;
        }
    }
    for (int i = 0; i < 6; i++) {
        if (current->hit_lights[i].count == 0) {
            current->hit_lights[i].node = cs;
            current->hit_lights[i].count = n;
            return;
        }
    }
}

void tm1_update_hit_lights(void)
{
    current->hit_light_flash = !current->hit_light_flash;
    for (int i = 0; i < 6; i++) {
        if (current->hit_lights[i].count > 0) {
            int remaining = --current->hit_lights[i].count;
            current->hit_lights[i].node->env = remaining && current->hit_light_flash ? &red_flash : NULL;
        }
    }
}
void UAeffectSetFreezeLight(Cs* cs)
{
    cs->env = &freeze_light;
}
void tm1_clear_freeze_light(u8 player, s16 index)
{
    if (current)
        (player ? GetPlayerCs3D(index) : GetAICs3D(index))->env = NULL;
}
void tm1_capture_car_lights(void)
{
    for (int i = 0; i <= current->opponent_count + current->duel; i++) {
        Cs* cs = current->duel || !i ? GetPlayerCs3D(i) : GetAICs3D(i - 1);
        current->displayed_lights[i] = cs->env ? cs->env->index : 0;
    }
}
void tm1_update_car_speed(int car_id, int speed)
{
    if (!current || car_id < 10 || car_id > 130)
        return;
    int slot = car_id / 10;
    if (car_id == 40) {
        current->wheel_steps[slot] -= speed / 7;
    } else {
        current->wheel_distance[slot] += abs(speed);
        if (current->wheel_distance[slot] >= 321) {
            current->wheel_distance[slot] = 0;
            current->wheel_steps[slot] += speed < 0 ? -1 : 1;
        }
    }
}
void tm1_wheel_steps(const CollisionScene* scene, int64_t* out)
{
    memcpy(out, scene->wheel_steps, sizeof scene->wheel_steps);
}
void tm1_car_lights(const CollisionScene* s, u8* out)
{
    memcpy(out, s->displayed_lights, sizeof s->displayed_lights);
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
        return 1;
    }
    return 0;
}

static void flame_burst(VECTOR3 pos)
{
    int frame = 1 + (arena_rand() & 3);
    int i = free_explosion();
    if (i < 0)
        return;
    current->bursts[i].pos = pos;
    current->bursts[i].life = 2;
    current->bursts[i].frame = frame;
    VECTOR3* vel = &current->bursts[i].vel;
    vel->vx = (arena_rand() & 7) - 3;
    vel->vy = (arena_rand() & 7) - 3;
    vel->vz = (arena_rand() & 15) > 5 ? 0 : (arena_rand() & 15) - 5;
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
size_t tm1_taser_arcs(CollisionScene* s, int view, VECTOR3 (*out)[16])
{
    size_t count = 0;
    for (size_t i = 0; i < 3; i++)
        if (s->tasers[i].visible[view])
            memcpy(out[count++], s->tasers[i].points[view], sizeof s->tasers[i].points[view]);
    return count;
}

static size_t copy_bullets(const CollisionScene* scene, s32 (*out)[8])
{
    size_t count = 0;
    for (size_t i = 0; i < 40; i++) {
        const Bullet* b = &scene->bullets[i];
        if (b->life > 0) {
            s32 pose[] = { b->pos.vx, b->pos.vy, b->pos.vz, b->prev.vx, b->prev.vy,
                b->prev.vz, b->kind, b->life };
            memcpy(out[count++], pose, sizeof pose);
        }
    }
    return count;
}

void arena_capture_bullets(void)
{
    current->drawn_bullet_count = copy_bullets(current, current->drawn_bullets);
}

size_t tm1_drawn_bullets(CollisionScene* scene, int second_view, s32 (*out)[8])
{
    if (second_view)
        return copy_bullets(scene, out);
    memcpy(out, scene->drawn_bullets, scene->drawn_bullet_count * sizeof(*out));
    return scene->drawn_bullet_count;
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
