// Adapted from JP TARGETS.c move_mercs and its UAEFFECT.c site tables.
#include "tm1/arena_guns.h"

static const s32 warehouse_sites[][2] = {
    {-1782, 882},
    {-1782, -300},
    {-1518, 336},
    {-1200, 300},
    {-1200, -887},
    {-18, 882},
    {-608, -18},
    {-608, -618},
};

static const s32 freeway_sites[][2] = {
    {-2082, -714},
    {-984, 1140},
    {-1032, -1140},
    {888, 1140},
    {1080, -1140},
    {2082, 442},
};

static const s32 park_sites[][2] = {
    {-580, -18},
    {-580, -518},
    {0, 582},
    {0, -300},
    {372, 300},
    {600, 48},
    {276, -877},
    {1200, 582},
    {1200, -18},
    {-608, -18},
    {1787, 882},
    {1787, -618},
    {0, -494},
    {1768, -494},
};

static const s32 cyburbia_sites[][2] = {
    {-282, 52},
    {144, 2040},
    {12, 1200},
    {600, 600},
    {1200, 1765},
    {1200, 20},
    {2036, -240},
    {2400, 1200},
    {2756, 2040},
    {3000, 12},
    {3588, 1788},
    {3882, 1422},
};
static const s32 rooftop_sites[][2] = {
    {550, 550},
    {850, 1275},
    {900, 550},
    {50, 1000},
    {1050, 1025},
};

static const s32 rooftop_heights[5] = {
    0x3CA,
    0x384,
    0x3E8,
    0x366,
    0x3E8,
};


void arena_move_mercs(ArenaMerc* hovermerc, int level, const VECTOR3* player)
{
    VECTOR3 pp = *player;
    VECTOR3 aim;
    const s32 (*table)[2];
    s32 count;
    s32 last;
    s32 i;
    s32 j;
    s32 k;
    s32 best;
    s32 d;
    s32 r;
    s32 st;
    u8 firing;
    u8 launching;

    firing = 0;
    launching = 0;
    switch (level) {
    case LEVEL2_WAREHOUSE:
        table = warehouse_sites;
        count = 8;
        i = 1;
        last = 2;
        break;
    case LEVEL3_FREEWAY:
        table = freeway_sites;
        count = 6;
        i = 0;
        last = 1;
        break;
    case LEVEL4_CITY_PARK:
        table = park_sites;
        count = 14;
        i = 1;
        last = 2;
        break;
    case LEVEL5_CYBURBIA:
        table = cyburbia_sites;
        count = 12;
        i = 0;
        last = 1;
        break;
    case LEVEL6_ROOFTOPS:
        table = rooftop_sites;
        count = 5;
        i = 0;
        last = 2;
        break;
    default:
        return;
    }
    for (; i < last; i++) {
        ArenaMerc* merc = &hovermerc[i];
        merc->hover_sound = 0;
        if (!merc->node)
            continue;
        st = merc->phase;
        if (st == 0) {
            merc->timer--;
            if (merc->timer <= 0) {
                merc->phase = 1;

                best = abs(table[0][0] * 8 - pp.vx) + abs(table[0][1] * 8 - pp.vy);
                k = 0;
                for (j = 1; j < count; j++) {
                    d = abs(table[j][0] * 8 - pp.vx) + abs(table[j][1] * 8 - pp.vy);
                    if (d < best && hovermerc[1 - i].site != j) {
                        best = d;
                        k = j;
                    }
                }
                merc->site = k;
                merc->model.pos.vx = table[k][0] * 8;
                merc->model.pos.vy = table[k][1] * 8;
                merc->model.pos.vz = 400;
                merc->altitude = 0;
                if (level == LEVEL6_ROOFTOPS) {
                    merc->model.pos.vz += rooftop_heights[k] * 8;
                    merc->altitude = rooftop_heights[k] * 8;
                }
                merc->timer = -1;
                merc->node->sel = 0;
                merc->model.drawMode = 1;
            }
        } else if (st == 1) {
            if (merc->altitude >= merc->model.pos.vz) {
                merc->phase = 2;
                merc->node->sel = 1;
                merc->timer = 16;
            } else {
                merc->model.pos.vz -= 4;
            }
        } else if (st == 2) {
            merc->timer--;
            if (merc->timer <= 0) {
                merc->node->sel = 2;
                merc->phase = 3;
                r = arena_rand() & 7;
                if (i != 0) {
                    merc->timer = r + 10;
                } else {
                    merc->timer = r + 18;
                }
                merc->shots = 0;
            }
        } else if (st == 3) {
            merc->timer--;
            if (merc->timer > 0) {
                if (merc->timer < 8) {
                    merc->node->sel = 2;
                }
                continue;
            }

            if (merc->model.epNode->h.kind == UA_HOVER_COP) {
                if (merc->shots < 5) {
                    merc->model.pos.vz += 48;
                    firing = 1;
                    aim.vx = pp.vx - merc->model.pos.vx + arena_rand() % 200
                        - (200 >> 1);
                    aim.vy = pp.vy - merc->model.pos.vy + arena_rand() % 200
                        - (200 >> 1);
                    aim.vz = pp.vz - merc->model.pos.vz;
                    s_create_bullet(
                        -merc->model.epNode->h.kind, &aim, &merc->model.pos, 2, 1, 28, 0);
                    merc->node->sel = 3;
                    merc->timer = (arena_rand() & 0x1F) + 10;
                    merc->shots++;
                    if (merc->shots == 5) {
                        merc->timer += 40 + (arena_rand() & 0x1F);
                    }
                    arena_flash(&merc->model.pos);
                    merc->model.pos.vz -= 48;
                    goto sound;
                }
            } else {
                k = merc->shots;
                if (level == LEVEL6_ROOFTOPS) {
                    if (k < 3) {
                        goto fire;
                    }
                } else if (k < 2) {
                    goto fire;
                }
            }
            merc->node->sel = 1;
            merc->phase = 4;
            goto sound;
        fire:
            merc->model.pos.vz += 62;
            aim.vx = 0;
            aim.vy = 0;
            aim.vz = ratan2(pp.vx - merc->model.pos.vx + arena_rand() % 200
                             - (200 >> 1),
                         pp.vy - merc->model.pos.vy)
                + arena_rand() % 200 - (200 >> 1);
            arena_cop_missile(-merc->model.epNode->h.kind, aim.vz, &merc->model.pos);
            arena_flash(&merc->model.pos);
            do_puff(&merc->model.pos);
            merc->model.pos.vz -= 62;
            merc->node->sel = 3;
            launching = 1;
            merc->timer = (arena_rand() & 0x3F) + 30;
            merc->shots++;
            if (merc->shots >= 2) {
                merc->timer += 40 + (arena_rand() & 0xF);
            }
        } else if (st == 4) {
            if (merc->model.pos.vz < merc->altitude + 400) {
                merc->model.pos.vz += 4;
            } else {
                merc->phase = 0;
                merc->timer = arena_rand() % 250 + (250 >> 1);
                merc->node->sel = 4;
                merc->site = -1;
            }
        }
    sound:
        merc->hover_sound = 1;
        if (firing)
            arena_effect(&merc->model.pos, 51);
        if (launching)
            arena_effect(&merc->model.pos, 52);
    }
}
