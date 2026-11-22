// Adapted from TARGETS.c. Coordinates and timing use PS1 simulation units.
#include "tm1/arena_guns.h"

static const PedPath pedPathLevel1[3] = {
    { -1230, 845, -1230, 895 },
    { -655, -168, -605, -168 },
    { -1745, -72, -1795, -72 },
};

static const PedPath pedPathLevel2[5] = {
    { -2172, -420, -2172, -348 },
    { -2125, -545, -2040, -545 },
    { -2125, -833, -2040, -833 },
    { 2038, 691, 2123, 691 },
    { 2038, 547, 2123, 547 },
};

static const PedPath pedPathLevel3[9] = {
    { -373, -846, -373, -891 },
    { -52, 134, 28, 134 },
    { 229, 846, 229, 896 },
    { 996, -648, 996, -528 },
    { 1252, -431, 1147, -431 },
    { 546, -625, 646, -625 },
    { 860, 345, 860, 250 },
    { 1148, -121, 1253, -121 },
    { 1333, -604, 1333, -654 },
};

static const PedPath pedPathLevel3b[1] = {
    { 756, 96, 826, -57 },
};

static const PedPath pedPathLevel4[14] = {
    { 65, 810, 65, 985 },
    { 1090, 1258, 1090, 1142 },
    { 812, 668, 984, 668 },
    { 1240, 1430, 1240, 1629 },
    { 1890, 1170, 1890, 1270 },
    { 3510, 1445, 3510, 1555 },
    { 3510, 235, 3510, 365 },
    { 1105, 1440, 1354, 1440 },
    { 3230, 1120, 3230, 1250 },
    { 3230, 680, 3365, 680 },
    { 537, 982, 663, 982 },
    { 130, 542, 130, 658 },
    { 1000, 254, 1000, 346 },
    { 1550, 1238, 1950, 1238 },
};

static const PedPath pedPathLevel5[2] = {
    { 10, 10, 590, 10 },
    { 1510, 10, 1965, 10 },
};

void arena_choose_ped_path(ArenaPed* ped, int level, int route, int end)
{
    const PedPath* table;
    s32 count;
    s32 n;
    s32 dx;
    s32 dy;

    switch (level) {
    case LEVEL2_WAREHOUSE:
        table = pedPathLevel1;
        count = 3;
        break;
    case LEVEL3_FREEWAY:
        table = pedPathLevel2;
        count = 5;
        break;
    case LEVEL4_CITY_PARK:
        table = pedPathLevel3;
        count = 9;
        if (ped->id == UA_UNK_773) {
            table = pedPathLevel3b;
            count = 1;
        }
        break;
    case LEVEL5_CYBURBIA:
        table = pedPathLevel4;
        count = 14;
        break;
    case LEVEL6_ROOFTOPS:
        table = pedPathLevel5;
        count = 2;
        break;
    default:
        return;
    }
    n = route % count;
    ped->path = &table[n];
    if (end & 1) {
        ped->direction = -1;
        ped->model.pos.vx = table[n].x1 * 8;
        ped->model.pos.vy = table[n].y1 * 8;
        ped->model.pos.vz = 0;
        dx = table[n].x0 - table[n].x1;
        dy = table[n].y0 - table[n].y1;
    } else {
        ped->direction = 1;
        ped->model.pos.vx = table[n].x0 * 8;
        ped->model.pos.vy = table[n].y0 * 8;
        ped->model.pos.vz = 0;
        dx = table[n].x1 - table[n].x0;
        dy = table[n].y1 - table[n].y0;
    }
    if (dx < 0) {
        ped->model.rot.vz = 0xC00;
    }
    if (dx > 0) {
        ped->model.rot.vz = 0x400;
    }
    if (dy < 0) {
        ped->model.rot.vz = 0x800;
    }
    if (level == LEVEL6_ROOFTOPS) {
        ped->model.pos.vz = 0x3CA;
    }
    ped->model.drawMode = 1;
    ped->ticks = 0;
    ped->node->sel = 1;
    ped->frame = 0;
    ped->state = 0;
}

static void select_frame(ArenaPed* p)
{
    if (p->frame < p->node->nChild)
        p->node->sel = p->frame;
}

void arena_ped_hit(ArenaPed* p, int yaw)
{
    if (p->id == 760) return;
    arena_effect(&p->model.pos, 36);
    p->state = 2;
    if (p->hit_end > p->alert_end) p->frame = p->alert_end + 1;
    p->ticks = 0;
    select_frame(p);
    p->model.pos.vz += 4;
    setVector(&p->model.rot, 0, 0, yaw);
    RotMatrixYXZ(&p->model.rot, &p->model.mat);
}

static int step_towards(int pos, int target)
{
    if (abs(target - pos) < 6) return target;
    return pos + (pos < target ? 6 : -6);
}

void arena_move_ped(ArenaPed* p, int level, const VECTOR3* player)
{
    VECTOR3* pos = &p->model.pos;
    if (p->state == 2) {
        VECTOR3 forward = {0, 24, 0}, offset;
        mathMulTransVec(&p->model.mat, &forward, &offset);
        pos->vx += offset.vx;
        pos->vy += offset.vy;
        pos->vz += 8;
        if (p->ticks++ > 8) {
            if (p->frame >= p->hit_end) {
                p->state = 3;
                p->cooldown = 240;
                p->model.drawMode = 0;
            } else {
                p->frame++;
            }
            p->ticks = 0;
            select_frame(p);
        }
    }
    if (p->state == 3 && --p->cooldown <= 0) {
        int route = arena_rand();
        arena_choose_ped_path(p, level, route, arena_rand());
        RotMatrixYXZ(&p->model.rot, &p->model.mat);
    }
    if (++p->ticks < 4) return;
    int dx = abs(pos->vx - player->vx), dy = abs(pos->vy - player->vy), dz = abs(pos->vz - player->vz);
    if (p->state == 0 && p->id != 760 && dx < 480 && dy < 480 && dz < 480) {
        p->state = 1;
        if (p->walk_end < p->alert_end) p->frame = p->walk_end + 1;
        p->ticks = 0;
        select_frame(p);
    }
    if (p->state == 0) {
        int x = (p->direction == 1 ? p->path->x1 : p->path->x0) * 8;
        int y = (p->direction == 1 ? p->path->y1 : p->path->y0) * 8;
        p->ticks = 0;
        if (++p->frame > p->walk_end) p->frame = 0;
        select_frame(p);
        if (pos->vx == x && pos->vy == y) {
            p->direction = -p->direction;
            p->model.rot.vz = (p->model.rot.vz + 2048) & 4095;
            RotMatrixYXZ(&p->model.rot, &p->model.mat);
        }
        pos->vx = step_towards(pos->vx, x);
        pos->vy = step_towards(pos->vy, y);
    } else if (p->state == 1) {
        if (dx < 960 && dy < 960 && dz < 960) {
            p->ticks = 1;
            if (p->frame < p->alert_end) p->frame++;
            select_frame(p);
            if ((p->id == 751 || p->id == 756) && !(arena_rand() & 15)) {
                VECTOR3 muzzle = *pos;
                muzzle.vz += 30;
                VECTOR3 aim = {player->vx - muzzle.vx, player->vy - muzzle.vy, player->vz - muzzle.vz};
                create_bullet(-p->id, &aim, &muzzle, 1);
                arena_effect(&muzzle, 53);
            }
        } else {
            p->state = p->ticks = p->frame = p->node->sel = 0;
        }
    }
}
