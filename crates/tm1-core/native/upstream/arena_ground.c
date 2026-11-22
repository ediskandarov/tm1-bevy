#include "tm1/arena_ground.h"
#include "tm1/arena_ai.h"

s16 numTriggerPoints, numBridges, numCheckBridges;
s16 numPotHoles, numCheckPotHoles, numCurbs, numSlickSpots;
TriggerPt tPoints[175];
TriggerPtGroups triggerPtGroups;
TriggerPtStartPts triggerPtStartPts;
Bridge bridges[59];
PotHole potHoles[5];
Curb curbs[25];
SlickSpot slickSpots[1];

void InitLevel2TriggerPoints(void)
{
    s16 i;
    s16 j;

    numTriggerPoints = 18;
    for (i = 0; i < numTriggerPoints; i++) {
        tPoints[i].f1A = 1;
        tPoints[i].type = 0;
        tPoints[i].f08 = 0;
        for (j = 0; j < 4; j++) {
            tPoints[i].link[j] = 0;
            tPoints[i].dist[j] = 0xA0;
        }
    }
    triggerPtStartPts.n = 3;
    for (i = 0; i < 15; i++) {
        triggerPtStartPts.start[i] = i;
        triggerPtStartPts.grp[i] = 0;
    }
    triggerPtGroups.count = 1;
    triggerPtGroups.pos[0].vx = -0x3E80;
    triggerPtGroups.pos[0].vy = -0x1F40;
    triggerPtGroups.pos[0].vz = 0;
    triggerPtGroups.w[0] = 0x2EE0;
    triggerPtGroups.d[0] = 0x3E80;
    for (i = 0; i < triggerPtGroups.count; i++) {
        for (j = 0; j < triggerPtGroups.count; j++) {
            triggerPtGroups.a[i][j] = -1;
            triggerPtGroups.b[i][j] = 0;
            triggerPtGroups.minSpeed[i][j] = 0x3C;
        }
    }
    tPoints[0].vx = -0x3840;
    tPoints[0].vy = -0x1C20;
    tPoints[0].f1A = 2;
    tPoints[0].link[0] = 3;
    tPoints[0].link[1] = 0xB;
    tPoints[1].vx = -0x2580;
    tPoints[1].vy = 0x960;
    tPoints[1].f1A = 3;
    tPoints[1].link[0] = 7;
    tPoints[1].link[1] = 8;
    tPoints[1].link[2] = 0x10;
    tPoints[2].vx = -0x12C0;
    tPoints[2].vy = 0x1C20;
    tPoints[2].f1A = 2;
    tPoints[2].link[0] = 0x11;
    tPoints[2].link[1] = 0xF;
    tPoints[3].vx = -0x3840;
    tPoints[3].vy = -0x960;
    tPoints[3].f1A = 3;
    tPoints[3].link[0] = 0;
    tPoints[3].link[1] = 4;
    tPoints[3].link[2] = 9;
    tPoints[4].vx = -0x3840;
    tPoints[4].vy = 0x960;
    tPoints[4].f1A = 3;
    tPoints[4].link[0] = 3;
    tPoints[4].link[1] = 5;
    tPoints[4].link[2] = 7;
    tPoints[5].vx = -0x3840;
    tPoints[5].vy = 0x1C20;
    tPoints[5].f1A = 2;
    tPoints[5].link[0] = 4;
    tPoints[5].link[1] = 6;
    tPoints[6].vx = -0x2F70;
    tPoints[6].vy = 0x1C20;
    tPoints[6].f1A = 3;
    tPoints[6].link[0] = 5;
    tPoints[6].link[1] = 7;
    tPoints[6].link[2] = 0x11;
    tPoints[7].vx = -0x2F70;
    tPoints[7].vy = 0x960;
    tPoints[7].f1A = 3;
    tPoints[7].link[0] = 4;
    tPoints[7].link[1] = 6;
    tPoints[7].link[2] = 1;
    tPoints[8].vx = -0x2580;
    tPoints[8].vy = -0x90;
    tPoints[8].f1A = 3;
    tPoints[8].link[0] = 1;
    tPoints[8].link[1] = 9;
    tPoints[8].link[2] = 0xE;
    tPoints[9].vx = -0x2580;
    tPoints[9].vy = -0x960;
    tPoints[9].f1A = 3;
    tPoints[9].link[0] = 3;
    tPoints[9].link[1] = 8;
    tPoints[9].link[2] = 0xA;
    tPoints[10].vx = -0x2580;
    tPoints[10].vy = -0x1350;
    tPoints[10].f1A = 3;
    tPoints[10].link[0] = 9;
    tPoints[10].link[1] = 0xB;
    tPoints[10].link[2] = 0xD;
    tPoints[11].vx = -0x2580;
    tPoints[11].vy = -0x1C20;
    tPoints[11].f1A = 3;
    tPoints[11].link[0] = 0;
    tPoints[11].link[1] = 0xA;
    tPoints[11].link[2] = 0xC;
    tPoints[12].vx = -0x12C0;
    tPoints[12].vy = -0x1C20;
    tPoints[12].f1A = 2;
    tPoints[12].link[0] = 0xB;
    tPoints[12].link[1] = 0xD;
    tPoints[13].vx = -0x12C0;
    tPoints[13].vy = -0x1350;
    tPoints[13].f1A = 3;
    tPoints[13].link[0] = 0xA;
    tPoints[13].link[1] = 0xC;
    tPoints[13].link[2] = 0xE;
    tPoints[14].vx = -0x12C0;
    tPoints[14].vy = -0x90;
    tPoints[14].f1A = 3;
    tPoints[14].link[0] = 8;
    tPoints[14].link[1] = 0xD;
    tPoints[14].link[2] = 0xF;
    tPoints[15].vx = -0x12C0;
    tPoints[15].vy = 0x960;
    tPoints[15].f1A = 3;
    tPoints[15].link[0] = 2;
    tPoints[15].link[1] = 0xE;
    tPoints[15].link[2] = 0x10;
    tPoints[16].vx = -0x1B90;
    tPoints[16].vy = 0x960;
    tPoints[16].f1A = 3;
    tPoints[16].link[0] = 1;
    tPoints[16].link[1] = 0xF;
    tPoints[16].link[2] = 0x11;
    tPoints[17].vx = -0x1B90;
    tPoints[17].vy = 0x1C20;
    tPoints[17].f1A = 3;
    tPoints[17].link[0] = 2;
    tPoints[17].link[1] = 6;
    tPoints[17].link[2] = 0x10;
}

void InitLevel2Bridges(void)
{
    s16 i;
    s16 j;

    numBridges = 0xC;
    numCheckBridges = 0xA;

    for (i = 0; i <= numBridges; i++) {
        bridges[i].unk08 = 0;
        bridges[i].unk0A = 10;
        for (j = 0; j < 4; j++) {
            bridges[i].obj[j] = -1;
        }
    }

    bridges[0].rampDir = 0;
    bridges[0].minX = -9600;
    bridges[0].minZ = -1516;
    bridges[0].y0 = 0;
    bridges[0].y1 = 0x30;
    bridges[0].extentX = 0x60;
    bridges[0].extentZ = 0xB0;
    bridges[0].obj[0] = 1;
    bridges[0].unk0A = 0x1E;

    bridges[1].rampDir = 1;
    bridges[1].minX = -9600;
    bridges[1].minZ = -1340;
    bridges[1].y0 = 0x30;
    bridges[1].y1 = 0;
    bridges[1].extentX = 0x60;
    bridges[1].extentZ = 0xA8;
    bridges[1].obj[0] = 0;
    bridges[1].unk0A = 0x1E;

    bridges[2].rampDir = 0;
    bridges[2].minX = -9600;
    bridges[2].minZ = 0x194;
    bridges[2].y0 = 0;
    bridges[2].y1 = 0x30;
    bridges[2].extentX = 0x60;
    bridges[2].extentZ = 0xB0;
    bridges[2].obj[0] = 3;
    bridges[2].unk0A = 0x1E;

    bridges[3].rampDir = 1;
    bridges[3].minX = -9600;
    bridges[3].minZ = 0x244;
    bridges[3].y0 = 0x30;
    bridges[3].y1 = 0;
    bridges[3].extentX = 0x60;
    bridges[3].extentZ = 0xA8;
    bridges[3].obj[0] = 2;
    bridges[3].unk0A = 0x1E;

    bridges[4].rampDir = 2;
    bridges[4].minX = -8332;
    bridges[4].minZ = 0x8A0;
    bridges[4].y0 = 0;
    bridges[4].y1 = 0x30;
    bridges[4].extentX = 0xB0;
    bridges[4].extentZ = 0x60;
    bridges[4].obj[0] = 5;
    bridges[4].unk0A = 0x1E;

    bridges[5].rampDir = 3;
    bridges[5].minX = -8156;
    bridges[5].minZ = 0x8A0;
    bridges[5].y0 = 0x30;
    bridges[5].y1 = 0;
    bridges[5].extentX = 0xA8;
    bridges[5].extentZ = 0x60;
    bridges[5].obj[0] = 4;
    bridges[5].unk0A = 0x1E;

    bridges[6].rampDir = 0;
    bridges[6].minX = -12456;
    bridges[6].minZ = 0x1530;
    bridges[6].y0 = 8;
    bridges[6].y1 = 0x20;
    bridges[6].extentX = 0x90;
    bridges[6].extentZ = 0x48;
    bridges[6].obj[0] = 0xA;

    bridges[7].rampDir = 1;
    bridges[7].minX = -12456;
    bridges[7].minZ = 0x1608;
    bridges[7].y0 = 0x20;
    bridges[7].y1 = 8;
    bridges[7].extentX = 0x90;
    bridges[7].extentZ = 0x48;
    bridges[7].obj[0] = 0xA;

    bridges[8].rampDir = 2;
    bridges[8].minX = -8304;
    bridges[8].minZ = -5256;
    bridges[8].y0 = 8;
    bridges[8].y1 = 0x20;
    bridges[8].extentX = 0x48;
    bridges[8].extentZ = 0x90;
    bridges[8].obj[0] = 0xB;

    bridges[9].rampDir = 3;
    bridges[9].minX = -8088;
    bridges[9].minZ = -5256;
    bridges[9].y0 = 0x20;
    bridges[9].y1 = 8;
    bridges[9].extentX = 0x48;
    bridges[9].extentZ = 0x90;
    bridges[9].obj[0] = 0xB;

    bridges[10].rampDir = 0;
    bridges[10].minX = -12456;
    bridges[10].minZ = 0x1578;
    bridges[10].y0 = 0x20;
    bridges[10].y1 = 0x20;
    bridges[10].extentX = 0x90;
    bridges[10].extentZ = 0x90;
    bridges[10].obj[0] = 6;
    bridges[10].obj[1] = 7;
    bridges[10].unk08 = 1;

    bridges[11].rampDir = 2;
    bridges[11].minX = -8232;
    bridges[11].minZ = -5256;
    bridges[11].y0 = 0x20;
    bridges[11].y1 = 0x20;
    bridges[11].extentX = 0x90;
    bridges[11].extentZ = 0x90;
    bridges[11].obj[0] = 8;
    bridges[11].obj[1] = 9;
    bridges[11].unk08 = 2;
}

void InitLevel2Curbs(void)
{
    numCurbs = 8;
    curbs[0].x = -14112;
    curbs[0].z = -6912;
    curbs[0].w = 4224;
    curbs[0].h = 4224;
    curbs[0].t = 8;

    curbs[1].x = -14112;
    curbs[1].z = -2112;
    curbs[1].w = 4224;
    curbs[1].h = 4224;
    curbs[1].t = 8;

    curbs[2].x = -14112;
    curbs[2].z = 2688;
    curbs[2].w = 1824;
    curbs[2].h = 4224;
    curbs[2].t = 8;

    curbs[3].x = -11984;
    curbs[3].z = 2688;
    curbs[3].w = 4800;
    curbs[3].h = 4224;
    curbs[3].t = 8;

    curbs[4].x = -6912;
    curbs[4].z = 2688;
    curbs[4].w = 1824;
    curbs[4].h = 4224;
    curbs[4].t = 8;

    curbs[5].x = -9312;
    curbs[5].z = 288;
    curbs[5].w = 4224;
    curbs[5].h = 1824;
    curbs[5].t = 8;

    curbs[6].x = -9312;
    curbs[6].z = -4800;
    curbs[6].w = 4224;
    curbs[6].h = 4800;
    curbs[6].t = 8;

    curbs[7].x = -9312;
    curbs[7].z = -6912;
    curbs[7].w = 4224;
    curbs[7].h = 1824;
    curbs[7].t = 8;
}

void InitLevel2PotHoles()
{
    s16 i;
    s16 t;
    s32 n;

    numPotHoles = 3;
    numCheckPotHoles = 3;
    i = 0;
    do {
        potHoles[i].active = 0;
        t = i + 1;
        do {
        } while (0);
        n = numPotHoles;
        i = t;
    } while (t < n);

    potHoles[0].x = -13928;
    potHoles[0].z = -6728;
    potHoles[0].a = 0x38;
    potHoles[0].b = 0xF00;
    potHoles[0].c = 0x18;

    potHoles[1].x = -13928;
    potHoles[1].z = -6728;
    potHoles[1].a = 0xF00;
    potHoles[1].b = 0x38;
    potHoles[1].c = 0x18;

    potHoles[2].x = -10128;
    potHoles[2].z = -6728;
    potHoles[2].a = 0x38;
    potHoles[2].b = 0x600;
    potHoles[2].c = 0x18;
}

void InitLevel1TriggerPoints(void)
{
    s16 i;
    s16 j;

    numTriggerPoints = 21;
    for (i = 0; i < numTriggerPoints; i++) {
        tPoints[i].f1A = 1;
        tPoints[i].type = 0;
        tPoints[i].f08 = 0;
        for (j = 0; j < 4; j++) {
            tPoints[i].link[j] = 0;
            tPoints[i].dist[j] = 0x77;
        }
    }
    triggerPtStartPts.n = 3;
    for (i = 0; i < 15; i++) {
        triggerPtStartPts.start[i] = i;
        triggerPtStartPts.grp[i] = 0;
    }
    triggerPtGroups.count = 1;
    triggerPtGroups.pos[0].vx = -0x1F40;
    triggerPtGroups.pos[0].vy = -0x1F40;
    triggerPtGroups.pos[0].vz = 0;
    triggerPtGroups.w[0] = 0x5DC0;
    triggerPtGroups.d[0] = 0x5DC0;
    for (i = 0; i < triggerPtGroups.count; i++) {
        for (j = 0; j < triggerPtGroups.count; j++) {
            triggerPtGroups.a[i][j] = -1;
            triggerPtGroups.b[i][j] = 0;
            triggerPtGroups.minSpeed[i][j] = 0x3C;
        }
    }
    tPoints[0].vx = 0xDC8;
    tPoints[0].vy = 0xE08;
    tPoints[0].f1A = 3;
    tPoints[0].link[0] = 2;
    tPoints[0].link[1] = 1;
    tPoints[0].link[2] = 7;
    tPoints[0].link[3] = 0xA;
    tPoints[1].vx = 0xB98;
    tPoints[1].vy = 0x1000;
    tPoints[1].f1A = 3;
    tPoints[1].link[0] = 0;
    tPoints[1].link[1] = 3;
    tPoints[1].link[2] = 4;
    tPoints[2].vx = 0x1000;
    tPoints[2].vy = 0xB50;
    tPoints[2].f1A = 3;
    tPoints[2].link[0] = 0xF;
    tPoints[2].link[1] = 0;
    tPoints[2].link[2] = 0xB;
    tPoints[3].vx = 0x728;
    tPoints[3].vy = 0x1000;
    tPoints[3].f1A = 3;
    tPoints[3].link[0] = 1;
    tPoints[3].link[1] = 8;
    tPoints[3].link[2] = 4;
    tPoints[4].vx = 0x850;
    tPoints[4].vy = 0xDC0;
    tPoints[4].f1A = 4;
    tPoints[4].link[0] = 3;
    tPoints[4].link[1] = 1;
    tPoints[4].link[2] = 6;
    tPoints[4].link[3] = 7;
    tPoints[5].vx = 0x2C0;
    tPoints[5].vy = 0xC88;
    tPoints[5].f1A = 3;
    tPoints[5].link[0] = 8;
    tPoints[5].link[1] = 0xC;
    tPoints[5].link[2] = 9;
    tPoints[6].vx = 0x728;
    tPoints[6].vy = 0xB50;
    tPoints[6].f1A = 4;
    tPoints[6].link[0] = 8;
    tPoints[6].link[1] = 4;
    tPoints[6].link[2] = 9;
    tPoints[6].link[3] = 0xA;
    tPoints[7].vx = 0xB98;
    tPoints[7].vy = 0xB98;
    tPoints[7].f1A = 4;
    tPoints[7].link[0] = 0;
    tPoints[7].link[1] = 4;
    tPoints[7].link[2] = 0xA;
    tPoints[7].link[3] = 0xB;
    tPoints[8].vx = 0x4F0;
    tPoints[8].vy = 0xE08;
    tPoints[8].f1A = 3;
    tPoints[8].link[0] = 3;
    tPoints[8].link[1] = 5;
    tPoints[8].link[2] = 6;
    tPoints[9].vx = 0x4F8;
    tPoints[9].vy = 0xA78;
    tPoints[9].f1A = 4;
    tPoints[9].link[0] = 5;
    tPoints[9].link[1] = 6;
    tPoints[9].link[2] = 0xC;
    tPoints[9].link[3] = 0xD;
    tPoints[10].vx = 0x9F0;
    tPoints[10].vy = 0x9F0;
    tPoints[10].f1A = 4;
    tPoints[10].link[0] = 6;
    tPoints[10].link[1] = 7;
    tPoints[10].link[2] = 0xD;
    tPoints[10].link[3] = 0xE;
    tPoints[11].vx = 0xDC8;
    tPoints[11].vy = 0x890;
    tPoints[11].f1A = 4;
    tPoints[11].link[0] = 7;
    tPoints[11].link[1] = 2;
    tPoints[11].link[2] = 0xE;
    tPoints[11].link[3] = 0xF;
    tPoints[12].vx = 0x2C0;
    tPoints[12].vy = 0x638;
    tPoints[12].f1A = 3;
    tPoints[12].link[0] = 5;
    tPoints[12].link[1] = 0x11;
    tPoints[12].link[2] = 9;
    tPoints[13].vx = 0x728;
    tPoints[13].vy = 0x728;
    tPoints[13].f1A = 4;
    tPoints[13].link[0] = 9;
    tPoints[13].link[1] = 0xA;
    tPoints[13].link[2] = 0x10;
    tPoints[13].link[3] = 0x11;
    tPoints[14].vx = 0xB98;
    tPoints[14].vy = 0x728;
    tPoints[14].f1A = 4;
    tPoints[14].link[0] = 0xA;
    tPoints[14].link[1] = 0xB;
    tPoints[14].link[2] = 0x10;
    tPoints[14].link[3] = 0x12;
    tPoints[15].vx = 0xFB8;
    tPoints[15].vy = 0x6E0;
    tPoints[15].f1A = 3;
    tPoints[15].link[0] = 0x12;
    tPoints[15].link[1] = 2;
    tPoints[15].link[2] = 0xB;
    tPoints[16].vx = 0xA78;
    tPoints[16].vy = 0x500;
    tPoints[16].f1A = 4;
    tPoints[16].link[0] = 0xD;
    tPoints[16].link[1] = 0xE;
    tPoints[16].link[2] = 0x13;
    tPoints[16].link[3] = 0x14;
    tPoints[17].vx = 0x4F0;
    tPoints[17].vy = 0x4B0;
    tPoints[17].f1A = 3;
    tPoints[17].link[0] = 0xC;
    tPoints[17].link[1] = 0x13;
    tPoints[17].link[2] = 0xD;
    tPoints[18].vx = 0xDC8;
    tPoints[18].vy = 0x4B0;
    tPoints[18].f1A = 3;
    tPoints[18].link[0] = 0x14;
    tPoints[18].link[1] = 0xF;
    tPoints[18].link[2] = 0xE;
    tPoints[19].vx = 0x728;
    tPoints[19].vy = 0x2C0;
    tPoints[19].f1A = 3;
    tPoints[19].link[0] = 0x14;
    tPoints[19].link[1] = 0x11;
    tPoints[19].link[2] = 0x10;
    tPoints[20].vx = 0xB98;
    tPoints[20].vy = 0x2C0;
    tPoints[20].f1A = 3;
    tPoints[20].link[0] = 0x13;
    tPoints[20].link[1] = 0x12;
    tPoints[20].link[2] = 0x10;
}

void InitLevel1Bridges(void)
{
    s16 i;
    s16 j;

    numBridges = 4;
    numCheckBridges = 4;

    for (i = 0; i <= numBridges; i++) {
        bridges[i].unk08 = 0;
        bridges[i].unk0A = 10;
        for (j = 0; j < 4; j++) {
            bridges[i].obj[j] = -1;
        }
    }

    bridges[0].rampDir = 0;
    bridges[0].minX = 0x270;
    bridges[0].minZ = 0x8B8;
    bridges[0].y0 = 0;
    bridges[0].y1 = 0x28;
    bridges[0].extentX = 0xA0;
    bridges[0].extentZ = 0xA8;
    bridges[0].obj[0] = 1;

    bridges[1].rampDir = 1;
    bridges[1].minX = 0x270;
    bridges[1].minZ = 0x960;
    bridges[1].y0 = 0x28;
    bridges[1].y1 = 0;
    bridges[1].extentX = 0xA0;
    bridges[1].extentZ = 0xB0;
    bridges[1].obj[0] = 0;

    bridges[2].rampDir = 0;
    bridges[2].minX = 0x7D0;
    bridges[2].minZ = 0x910;
    bridges[2].y0 = 0;
    bridges[2].y1 = 0x50;
    bridges[2].extentX = 0xA0;
    bridges[2].extentZ = 0x140;

    bridges[3].rampDir = 3;
    bridges[3].minX = 0x960;
    bridges[3].minZ = 0x640;
    bridges[3].y0 = 0x50;
    bridges[3].y1 = 0;
    bridges[3].extentX = 0x140;
    bridges[3].extentZ = 0xA0;
}

void InitLevel1PotHoles()
{
    s16 i;
    s16 t;
    s32 n;

    numPotHoles = 4;
    numCheckPotHoles = 4;
    i = 0;
    do {
        potHoles[i].active = 0;
        t = i + 1;
        do {
        } while (0);
        n = numPotHoles;
        i = t;
    } while (t < n);

    potHoles[0].x = 0x3C8;
    potHoles[0].z = 0x6D8;
    potHoles[0].a = 0x50;
    potHoles[0].b = 0xA0;
    potHoles[0].c = 0x18;

    potHoles[1].x = 0x438;
    potHoles[1].z = 0xC08;
    potHoles[1].a = 0xA0;
    potHoles[1].b = 0xA0;
    potHoles[1].c = 0x18;

    potHoles[2].x = 0x910;
    potHoles[2].z = 0x370;
    potHoles[2].a = 0x78;
    potHoles[2].b = 0x80;
    potHoles[2].c = 0x18;

    potHoles[3].x = 0xED8;
    potHoles[3].z = 0x870;
    potHoles[3].a = 0xA0;
    potHoles[3].b = 0xB8;
    potHoles[3].c = 0x18;
}

s32 SmoothValue(s32 from, s32 to, s32 pct)
{
    return (from * pct + to * (100 - pct)) / 100;
}

void CarInitBounceDeltas(Car* car, u8 isPlayer)
{
    CarBounce* bp;
    CarMotion* m;

    if (isPlayer) {
        bp = &PLAYER_CAR(car)->bounce;
        m = &PLAYER_CAR(car)->motion;
    } else {
        bp = &AI_CAR(car)->bounce;
        m = &AI_CAR(car)->motion;
    }

    if (m->vel.vy / 32) {
        bp->unk14 = bp->unk12 - RANDINT(bp->unk12 / 2);
        if (bp->unk18 > 0) {
            bp->unk24 = bp->unk18 - RANDINT(bp->unk18 / 2);
        } else if (bp->unk18 < 0) {
            bp->unk24 = bp->unk18 + RANDINT(bp->unk18 / 2);
        }
        if (bp->unk1C > 0) {
            bp->unk28 = bp->unk1C - RANDINT(bp->unk1C / 2);
        } else if (bp->unk1C < 0) {
            bp->unk28 = bp->unk1C + RANDINT(bp->unk1C / 2);
        }
    } else {
        bp->unk04 = 0;
    }
}

void SetBounce(Car* car, s32 level, s32 a, s32 b, u8 isPlayer)
{
    CarBounce* bp;
    u32 mag;
    u32 ang;

    if (isPlayer) {
        bp = &PLAYER_CAR(car)->bounce;
    } else {
        bp = &AI_CAR(car)->bounce;
    }

    bp->unk04 = level;
    bp->unk0C = b;
    if (bp->unk0C <= 0) {
        bp->unk00 = level;
        bp->unk08 = a;
    }
    bp->unk0E = arena_rand() % 11;

    switch (level) {
    case 5:
        mag = 0x71;
        ang = 0x200;
        break;
    case 4:
        mag = 0x44;
        ang = 0x200;
        break;
    case 3:
        mag = 0x2D;
        ang = 0x200;
        break;
    case 2:
        mag = 0x0B;
        ang = 0x155;
        break;
    case 1:

        mag = 2;
        ang = 0xE3;
        break;
    default:
        mag = 2;
        ang = 0xE3;
        break;
    }

    mag = mag - arena_rand() % (mag >> 1);
    if (a == 0 || a == 2) {
        if (arena_rand() & 1) {
            mag = -mag;
        }
        bp->unk18 = mag;
    } else {
        bp->unk18 = 0;
    }
    if (a == 1 || a == 2) {
        if (arena_rand() & 1) {
            mag = -mag;
        }
        bp->unk1C = mag;
    } else {
        bp->unk1C = 0;
    }
    bp->unk12 = ang - RANDINT(ang >> 1);
    CarInitBounceDeltas(car, isPlayer);
}

void UpdateBounce(Car* car, u8 isPlayer)
{
    CarBounce* bp;
    CarMotion* m;
    CarStats* st;
    s32 d;
    s32 q;
    s16 v;
    s32 qs;
    s32 amp[4];

    if (isPlayer) {
        bp = &PLAYER_CAR(car)->bounce;
        m = &PLAYER_CAR(car)->motion;
        st = &PLAYER_CAR(car)->stats;
    } else {
        bp = &AI_CAR(car)->bounce;
        m = &AI_CAR(car)->motion;
        st = &AI_CAR(car)->stats;
    }

    if (isPlayer) {
        d = st->unkD4;
    } else {
        d = st->unkD4 / 10;
    }

    q = m->rotDelta.vz * st->unkF4 / d;
    qs = (s16)q;
    v = q;
    if (abs(st->unkF4) < abs(qs)) {
        if (st->unkF4 > 0) {
            if (qs > 0) {
                v = st->unkF4;
            } else {
                v = -st->unkF4;
            }
        } else {
            if (qs > 0) {
                v = -st->unkF4;
            } else {
                v = st->unkF4;
            }
        }
    }
    bp->unk34 = SmoothValue(bp->unk34, v, 90);

    if (isPlayer && PLAYER_CAR(car)->unkF4 == VR_MODE_UNK_1) {
        bp->unk16 = SmoothValue(bp->unk16, PLAYER_CAR(car)->speedDelta, 90);
        bp->unk30 = -bp->unk16 / 2;
    } else {
        bp->unk30 = 0;
    }

    if (bp->unk04) {
        bp->unk0E = bp->unk0E + bp->unk14;
        if (bp->unk0E < 4096) {
            amp[0] = bp->unk24 * m->vel.vy / (m->vel.vy < st->unkA8 ? st->unkA8 : m->vel.vy);
            amp[1] = bp->unk28 * m->vel.vy / (m->vel.vy < st->unkA8 ? st->unkA8 : m->vel.vy);
            bp->unk30 += rsin(bp->unk0E) * amp[0] / 4096;
            bp->unk34 += rsin(bp->unk0E) * amp[1] / 4096;
        } else {
            bp->unk0E = bp->unk0E - 4096;
            if (bp->unk0C > 0) {
                bp->unk0C--;
                bp->unk24 = bp->unk24 / 2;
                bp->unk28 = bp->unk28 / 2;
            } else {
                SetBounce(car, bp->unk00, bp->unk08, 0, isPlayer);
            }
        }
    } else if (bp->unk00) {
        SetBounce(car, bp->unk00, bp->unk08, 0, isPlayer);
    }
}

void InitTireInfo(CarTire* tires)
{
    tires[0].unk0 = 0;
    tires[0].unk1 = 0;
    tires[0].unk2 = 0;
    tires[0].catapulted = 0;
    tires[0].bombDamaged = 0;
    tires[1].unk0 = 0;
    tires[1].unk1 = 0;
    tires[1].unk2 = 0;
    tires[1].catapulted = 0;
    tires[1].bombDamaged = 0;
    tires[2].unk0 = 0;
    tires[2].unk1 = 0;
    tires[2].unk2 = 0;
    tires[2].catapulted = 0;
    tires[2].bombDamaged = 0;
    tires[3].unk0 = 0;
    tires[3].unk1 = 0;
    tires[3].unk2 = 0;
    tires[3].catapulted = 0;
    tires[3].bombDamaged = 0;
    tires[0].unk22 = 0;
    tires[1].unk22 = 0;
    tires[2].unk22 = 0;
    tires[3].unk22 = 0;
    tires[0].unk24 = 0;
    tires[1].unk24 = 0;
    tires[2].unk24 = 0;
    tires[3].unk24 = 0;
    tires[0].unk10 = 0;
    tires[1].unk10 = 0;
    tires[2].unk10 = 0;
    tires[3].unk10 = 0;
    tires[0].unk40 = 0;
    tires[0].unk16 = 0;
    tires[0].unk18 = 0;
    tires[0].unk1C = 0;
    tires[0].unk1E = 0;
    tires[0].unk20 = 0;
    tires[0].unk1A = 0;
    tires[0].unk26 = 2;
    tires[1].unk40 = 0;
    tires[1].unk16 = 0;
    tires[1].unk18 = 0;
    tires[1].unk1C = 0;
    tires[1].unk1E = 0;
    tires[1].unk20 = 0;
    tires[1].unk1A = 0;
    tires[1].unk26 = 2;
    tires[2].unk40 = 0;
    tires[2].unk16 = 0;
    tires[2].unk18 = 0;
    tires[2].unk1C = 0;
    tires[2].unk1E = 0;
    tires[2].unk20 = 0;
    tires[2].unk1A = 0;
    tires[2].unk26 = 2;
    tires[3].unk40 = 0;
    tires[3].unk16 = 0;
    tires[3].unk18 = 0;
    tires[3].unk1C = 0;
    tires[3].unk1E = 0;
    tires[3].unk20 = 0;
    tires[3].unk1A = 0;
    tires[3].unk26 = 2;
    tires[0].unk08 = 0;
    tires[0].unk0A = 0;
    tires[0].unk0E = 0;
    tires[0].unk3 = 0;
    tires[0].unk28 = 0;
    tires[0].unk12 = 0;
    tires[0].unk14 = 0;
    tires[0].unk40 = 0;
    tires[1].unk08 = 0;
    tires[1].unk0A = 0;
    tires[1].unk0E = 0;
    tires[1].unk3 = 0;
    tires[1].unk28 = 0;
    tires[1].unk12 = 0;
    tires[1].unk14 = 0;
    tires[1].unk40 = 0;
    tires[2].unk08 = 0;
    tires[2].unk0A = 0;
    tires[2].unk0E = 0;
    tires[2].unk3 = 0;
    tires[2].unk28 = 0;
    tires[2].unk12 = 0;
    tires[2].unk14 = 0;
    tires[2].unk40 = 0;
    tires[3].unk08 = 0;
    tires[3].unk0A = 0;
    tires[3].unk0E = 0;
    tires[3].unk3 = 0;
    tires[3].unk28 = 0;
    tires[3].unk12 = 0;
    tires[3].unk14 = 0;
    tires[3].unk40 = 0;
}

void InitTireGroup(CarTire* tires, s16 group)
{
    tires[0].unk10 = group;
    tires[1].unk10 = group;
    tires[2].unk10 = group;
    tires[3].unk10 = group;
    tires[0].unk40 = arena_group_height(group);
    tires[1].unk40 = arena_group_height(group);
    tires[2].unk40 = arena_group_height(group);
    tires[3].unk40 = arena_group_height(group);
    tires[0].unk22 = tires[0].unk40;
    tires[1].unk22 = tires[1].unk40;
    tires[2].unk22 = tires[2].unk40;
    tires[3].unk22 = tires[3].unk40;
    tires[0].unk24 = tires[0].unk40;
    tires[1].unk24 = tires[1].unk40;
    tires[2].unk24 = tires[2].unk40;
    tires[3].unk24 = tires[3].unk40;
    tires[0].unk0 = 1;
    tires[1].unk0 = 1;
    tires[2].unk0 = 1;
    tires[3].unk0 = 1;
}

s16 GetClosestTriggerPt(Car* car, u8 isPlayer)
{
    s32 group;
    s16 i;
    s16 best;
    s32 bestDist;
    s32 dx;
    s32 dz;
    s32 distance;
    CarMotion* m;

    if (isPlayer) {
        group = PLAYER_CAR(car)->unkC0;
        m = &PLAYER_CAR(car)->motion;
    } else {
        group = AI_CAR(car)->unk176;
        m = &AI_CAR(car)->motion;
    }
    best = 0;
    bestDist = 0x7FFFFFFF;
    for (i = 0; i < numTriggerPoints; i++) {
        if (tPoints[i].type != group && group < 11) {
            continue;
        }
        dx = m->pos.vx - tPoints[i].vx;
        dz = m->pos.vy - tPoints[i].vy;
        distance = abs(dx) + abs(dz);
        if (distance < bestDist) {
            best = i;
            bestDist = distance;
        }
    }
    return best;
}

void CalcTireCoordinates(Car* car, u8 isPlayer)
{
    CarMotion* m;
    CarStats* st;
    CarTire* tires;
    s16 g0;
    s16 g1;
    s16 g2;
    s16 g3;
    s32 stepLimit;
    s32 previousHeight;
    s32 value;
    s32 tireOffset;
    s32 previousGroup;
    VECTOR3 vin;
    VECTOR3 vout;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        st = &PLAYER_CAR(car)->stats;
        tires = PLAYER_CAR(car)->tires;
    } else {
        m = &AI_CAR(car)->motion;
        st = &AI_CAR(car)->stats;
        tires = AI_CAR(car)->tires;
    }
    vin.vx = -st->unk74 / 2;
    vin.vy = st->unk80;
    vin.vz = -st->unk6C;
    mathMulTransVec(&m->mat2, &vin, &tires[0].unk2C);
    tires[0].unk38 = m->pos.vx + tires[0].unk2C.vx;
    tires[0].unk3C = m->pos.vy + tires[0].unk2C.vy;
    vin.vx = st->unk74 / 2;
    vin.vy = st->unk80;
    vin.vz = -st->unk6C;
    mathMulTransVec(&m->mat2, &vin, &tires[1].unk2C);
    tires[1].unk38 = m->pos.vx + tires[1].unk2C.vx;
    tires[1].unk3C = m->pos.vy + tires[1].unk2C.vy;
    vin.vx = -st->unk74 / 2;
    vin.vy = st->unk84;
    vin.vz = -st->unk6C;
    mathMulTransVec(&m->mat2, &vin, &tires[2].unk2C);
    tires[2].unk38 = m->pos.vx + tires[2].unk2C.vx;
    tires[2].unk3C = m->pos.vy + tires[2].unk2C.vy;
    vin.vx = st->unk74 / 2;
    vin.vy = st->unk84;
    vin.vz = -st->unk6C;
    mathMulTransVec(&m->mat2, &vin, &tires[3].unk2C);
    tires[3].unk38 = m->pos.vx + tires[3].unk2C.vx;
    tires[3].unk3C = m->pos.vy + tires[3].unk2C.vy;

    vin.vx = m->vel.vx / 32;
    vin.vy = m->vel.vy / 32;
    vin.vz = m->vel.vz / 32;
    mathMulTransVec(&m->mat, &vin, &vout);
    tires[0].unk38 += vout.vx;
    tires[0].unk3C += vout.vy;
    tires[0].unk40 += vout.vz;
    tires[1].unk38 += vout.vx;
    tires[1].unk3C += vout.vy;
    tires[1].unk40 += vout.vz;
    tires[2].unk38 += vout.vx;
    tires[2].unk3C += vout.vy;
    tires[2].unk40 += vout.vz;
    tires[3].unk38 += vout.vx;
    tires[3].unk3C += vout.vy;
    tires[3].unk40 += vout.vz;

    value = tires[0].unk38;
    g0 = GetGroupTestPointIsIn(value, tires[0].unk3C, m->pos.vz);
    g1 = GetGroupTestPointIsIn(tires[1].unk38, tires[1].unk3C, m->pos.vz);
    g2 = GetGroupTestPointIsIn(tires[2].unk38, tires[2].unk3C, m->pos.vz);
    g3 = GetGroupTestPointIsIn(tires[3].unk38, tires[3].unk3C, m->pos.vz);
    stepLimit = st->unk5C + abs(m->vel.vy / 32);

    if (tires[0].unk10 != g0) {
        previousGroup = tires[0].unk10;
        value = arena_group_height(g0);
        previousHeight = arena_group_height(previousGroup);
        tireOffset = tires[0].unk40 - value;
        value -= previousHeight;
        if (value > 0) {
            if ((-stepLimit < tireOffset || tires[1].unk10 == g0 || tires[2].unk10 == g0
                    || tires[3].unk10 == g0)
                && m->vel.vy != 0) {
                tires[0].unk10 = g0;
                if (tires[0].unk0 == 0) {
                    tires[0].unk0 = 1;
                    if (tires[0].unk08 < numCheckBridges) {
                        tires[0].unk08++;
                    } else {
                        tires[0].unk08--;
                    }
                }
                tires[0].unk26 = 0;
                tires[0].unk16 = tireOffset;
                tires[0].unk24 = arena_group_height(g0);
            }
        } else if (value < 0) {
            tires[0].unk10 = g0;
            tires[0].unk0 = 1;
            tires[0].unk14 = 0;
            tires[0].unk26 = 0;
            tires[0].unk16 = tireOffset;
            tires[0].unk24 = arena_group_height(g0);
            tires[0].unk3 = 1;
        } else {
            tires[0].unk10 = g0;
        }
    }
    if (tires[1].unk10 != g1) {
        previousGroup = tires[1].unk10;
        value = arena_group_height(g1);
        previousHeight = arena_group_height(previousGroup);
        tireOffset = tires[1].unk40 - value;
        value -= previousHeight;
        if (value > 0) {
            if ((-stepLimit < tireOffset || tires[0].unk10 == g1 || tires[2].unk10 == g1
                    || tires[3].unk10 == g1)
                && m->vel.vy != 0) {
                tires[1].unk10 = g1;
                if (tires[1].unk0 == 0) {
                    tires[1].unk0 = 1;
                    if (tires[1].unk08 < numCheckBridges) {
                        tires[1].unk08++;
                    } else {
                        tires[1].unk08--;
                    }
                }
                tires[1].unk26 = 0;
                tires[1].unk16 = tireOffset;
                tires[1].unk24 = arena_group_height(g1);
            }
        } else if (value < 0) {
            tires[1].unk10 = g1;
            tires[1].unk0 = 1;
            tires[1].unk14 = 0;
            tires[1].unk26 = 0;
            tires[1].unk16 = tireOffset;
            tires[1].unk24 = arena_group_height(g1);
            tires[1].unk3 = 1;
        } else {
            tires[1].unk10 = g1;
        }
    }
    if (tires[2].unk10 != g2) {
        previousGroup = tires[2].unk10;
        value = arena_group_height(g2);
        previousHeight = arena_group_height(previousGroup);
        tireOffset = tires[2].unk40 - value;
        value -= previousHeight;
        if (value > 0) {
            if ((-stepLimit < tireOffset || tires[0].unk10 == g2 || tires[1].unk10 == g2
                    || tires[3].unk10 == g2)
                && m->vel.vy != 0) {
                tires[2].unk10 = g2;
                if (tires[2].unk0 == 0) {
                    tires[2].unk0 = 1;
                    if (tires[2].unk08 < numCheckBridges) {
                        tires[2].unk08++;
                    } else {
                        tires[2].unk08--;
                    }
                }
                tires[2].unk26 = 0;
                tires[2].unk16 = tireOffset;
                tires[2].unk24 = arena_group_height(g2);
            }
        } else if (value < 0) {
            tires[2].unk10 = g2;
            tires[2].unk0 = 1;
            tires[2].unk14 = 0;
            tires[2].unk26 = 0;
            tires[2].unk16 = tireOffset;
            tires[2].unk24 = arena_group_height(g2);
            tires[2].unk3 = 1;
        } else {
            tires[2].unk10 = g2;
        }
    }
    if (tires[3].unk10 != g3) {
        previousGroup = tires[3].unk10;
        value = arena_group_height(g3);
        previousHeight = arena_group_height(previousGroup);
        tireOffset = tires[3].unk40 - value;
        value -= previousHeight;
        if (value > 0) {
            if (-stepLimit < tireOffset || tires[0].unk10 == g3 || tires[1].unk10 == g3
                || tires[2].unk10 == g3) {
                if (m->vel.vy != 0) {
                    tires[3].unk10 = g3;
                    if (tires[3].unk0 == 0) {
                        tires[3].unk0 = 1;
                        if (tires[3].unk08 < numCheckBridges) {
                            tires[3].unk08++;
                        } else {
                            tires[3].unk08--;
                        }
                    }
                    tires[3].unk26 = 0;
                    tires[3].unk16 = tireOffset;
                    tires[3].unk24 = arena_group_height(g3);
                }
            }
        } else if (value < 0) {
            tires[3].unk10 = g3;
            tires[3].unk0 = 1;
            tires[3].unk14 = 0;
            tires[3].unk26 = 0;
            tires[3].unk16 = tireOffset;
            tires[3].unk24 = arena_group_height(g3);
            tires[3].unk3 = 1;
        } else {
            tires[3].unk10 = g3;
        }
    }
    if (isPlayer) {
        if (g0 == g1 && g0 == g2 && g0 == g3) {
            PLAYER_CAR(car)->unkC0 = g0;
        }
    } else {
        AI_CAR(car)->unk176 = g0;
    }
}

void CheckBridges(Car* car, u8 isPlayer)
{
    CarMotion* m;
    CarStats* st;
    u8* flags;
    CarTire* tires;
    u8 padFlag;
    CarCollision* collision;
    s32 bearing;
    s32 d;
    s32 h;
    u8 ok;
    u8 any3;
    u8 any0;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        collision = &PLAYER_CAR(car)->collision;
        padFlag = PLAYER_CAR(car)->skid[0x15];
        st = &PLAYER_CAR(car)->stats;
        flags = PLAYER_CAR(car)->flags;
        tires = PLAYER_CAR(car)->tires;
        bearing = PLAYER_CAR(car)->bearing;
    } else {
        m = &AI_CAR(car)->motion;
        collision = &AI_CAR(car)->collision;
        bearing = AI_CAR(car)->bearing;
        padFlag = 0;
        st = &AI_CAR(car)->stats;
        flags = AI_CAR(car)->flags;
        tires = AI_CAR(car)->tires;
    }
    d = m->vel.vy / 32;
    if (flags[10] == 0) {
        h = st->unk5C + abs(d);
    } else {
        h = 0xA0;
    }

    if (tires[0].unk0) {
        if (GetBridgeGroundHeight(&tires[0], tires[0].unk08, h) == 0) {
            tires[0].unk22 = tires[0].unk24;
        }
        if (tires[0].unk3 != 0 && tires[0].unk08 < numCheckBridges) {
            CheckForBridge(&tires[0], 0, h);
        }
        if (tires[0].bombDamaged == 0 && tires[0].catapulted == 0) {
            if (d < (s16)tires[0].unk14) {
                tires[0].unk14 = d;
            } else if ((s16)tires[0].unk14 < -d) {
                tires[0].unk14 = -d;
            }
        }
        GetBridgeTireHeight(&tires[0], h);
        if (isPlayer && tires[0].unk3 == 0 && tires[0].unk26 < 2) {
            tires[0].unk14 = (bridges[tires[0].unk08].unk0A * m->vel.vy) / st->unkA8;
        }
        if ((tires[0].unk12 & 7) == 0) {
            tires[0].unk12++;
        }
    } else {
        tires[0].unk14 = -1;
        CheckForBridge(&tires[0], 0, h);
        if (tires[0].unk3 != 0) {
            GetBridgeTireHeight(&tires[0], h);
        }
    }

    if (tires[1].unk0) {
        if (GetBridgeGroundHeight(&tires[1], tires[1].unk08, h) == 0) {
            tires[1].unk22 = tires[1].unk24;
        }
        if (tires[1].unk3 != 0 && tires[1].unk08 < numCheckBridges) {
            CheckForBridge(&tires[1], 0, h);
        }
        if (tires[1].bombDamaged == 0 && tires[1].catapulted == 0) {
            if (d < (s16)tires[1].unk14) {
                tires[1].unk14 = d;
            } else if ((s16)tires[1].unk14 < -d) {
                tires[1].unk14 = -d;
            }
        }
        GetBridgeTireHeight(&tires[1], h);
        if (isPlayer && tires[1].unk3 == 0 && tires[1].unk26 < 2) {
            tires[1].unk14 = (bridges[tires[1].unk08].unk0A * m->vel.vy) / st->unkA8;
        }
        if ((tires[1].unk12 & 7) == 0) {
            tires[1].unk12++;
        }
    } else {
        tires[1].unk14 = -1;
        CheckForBridge(&tires[1], 0, h);
        if (tires[1].unk3 != 0) {
            GetBridgeTireHeight(&tires[1], h);
        }
    }

    if (tires[2].unk0) {
        if (GetBridgeGroundHeight(&tires[2], tires[2].unk08, h) == 0) {
            tires[2].unk22 = tires[2].unk24;
        }
        if (tires[2].unk3 != 0 && tires[2].unk08 < numCheckBridges) {
            CheckForBridge(&tires[2], 0, h);
        }
        if (tires[2].bombDamaged == 0 && tires[2].catapulted == 0) {
            if (d < (s16)tires[2].unk14) {
                tires[2].unk14 = d;
            } else if ((s16)tires[2].unk14 < -d) {
                tires[2].unk14 = -d;
            }
        }
        GetBridgeTireHeight(&tires[2], h);
        if (isPlayer && tires[2].unk3 == 0 && tires[2].unk26 < 2) {
            tires[2].unk14 = (bridges[tires[2].unk08].unk0A * m->vel.vy) / st->unkA8;
        }
    } else {
        tires[2].unk14 = -1;
        CheckForBridge(&tires[2], 0, h);
        if (tires[2].unk3 != 0) {
            GetBridgeTireHeight(&tires[2], h);
        }
    }

    if (tires[3].unk0) {
        if (GetBridgeGroundHeight(&tires[3], tires[3].unk08, h) == 0) {
            tires[3].unk22 = tires[3].unk24;
        }
        if (tires[3].unk3 != 0 && tires[3].unk08 < numCheckBridges) {
            CheckForBridge(&tires[3], 0, h);
        }
        if (tires[3].bombDamaged == 0 && tires[3].catapulted == 0) {
            if (d < (s16)tires[3].unk14) {
                tires[3].unk14 = d;
            } else if ((s16)tires[3].unk14 < -d) {
                tires[3].unk14 = -d;
            }
        }
        GetBridgeTireHeight(&tires[3], h);
        if (isPlayer && tires[3].unk3 == 0 && tires[3].unk26 < 2) {
            tires[3].unk14 = (bridges[tires[3].unk08].unk0A * m->vel.vy) / st->unkA8;
        }
    } else {
        tires[3].unk14 = -1;
        CheckForBridge(&tires[3], 0, h);
        if (tires[3].unk3 != 0) {
            GetBridgeTireHeight(&tires[3], h);
        }
    }

    if ((isPlayer && flags[23] == 0 && flags[11] == 0) || (!isPlayer && flags[23] == 0)) {
        if (isPlayer) {
            ok = !(abs(m->vel.vy) < st->unkAC && padFlag);
        } else {
            ok = flags[0] != 0;
        }
        if (ok) {
            if (m->rot.vx >= 57) {
                if (m->rot.vx >= 228) {
                    m->vel.vy = m->vel.vy + 3 * st->unkA0;
                } else if (m->rot.vx >= 114) {
                    m->vel.vy = m->vel.vy + 2 * st->unkA0;
                } else {
                    m->vel.vy = m->vel.vy + st->unkA0;
                }
            } else if (m->rot.vx < -56) {
                if (m->rot.vx < -227) {
                    m->vel.vy = m->vel.vy - 3 * st->unkA0;
                } else if (m->rot.vx < -113) {
                    m->vel.vy = m->vel.vy - 2 * st->unkA0;
                } else {
                    m->vel.vy = m->vel.vy - st->unkA0;
                }
            }
        }
    }
    if (isPlayer) {
        if (st->unkAC < abs(m->vel.vy)) {
            if (m->rot.vy >= 171) {
                if (m->vel.vy <= 0) {
                    if (m->rotDelta.vz < st->unkEC) {
                        m->rotDelta.vz = m->rotDelta.vz + st->unkCC / 2;
                    }
                } else if (-st->unkEC < m->rotDelta.vz) {
                    m->rotDelta.vz = m->rotDelta.vz - st->unkCC / 2;
                }
            } else if (m->rot.vy < -170) {
                if (m->vel.vy > 0) {
                    if (m->rotDelta.vz < st->unkEC) {
                        m->rotDelta.vz = m->rotDelta.vz + st->unkCC / 2;
                    }
                } else if (-st->unkEC < m->rotDelta.vz) {
                    m->rotDelta.vz = m->rotDelta.vz - st->unkCC / 2;
                }
            }
        }
    }

    any3 = 0;
    if (tires[0].unk3 != 0 || tires[1].unk3 != 0 || tires[2].unk3 != 0 || tires[3].unk3 != 0) {
        any3 = 1;
    }
    flags[11] = any3;
    if (flags[11]) {
        SetFullCarDrift(m);
    }
    any0 = 0;
    if (tires[0].unk0 != 0 || tires[1].unk0 != 0 || tires[2].unk0 != 0 || tires[3].unk0 != 0) {
        any0 = 1;
    }
    if (any0) {
        if (flags[10] == 0) {
            collision->unk12 = 0xE;
        }
    } else if (flags[10] != 0) {
        SetBounce(car, 3, 2, 1, isPlayer);
        collision->unk12 = 0x27;
    }
    flags[10] = any0;
    if (flags[10] == 0) {
        return;
    }

    if (tires[0].unk6 != 0) {
        if (tires[1].unk0 != 0) {
            if (tires[0].unk08 == tires[1].unk08) {
                tires[0].unk0 = 1;
            }
        } else if (tires[2].unk0 != 0) {
            if (tires[0].unk08 == tires[2].unk08) {
                tires[0].unk0 = 1;
            }
        } else if (tires[3].unk0 != 0) {
            if (tires[0].unk08 == tires[3].unk08) {
                tires[0].unk0 = 1;
            }
        }
    }
    if (tires[1].unk6 != 0) {
        if (tires[0].unk0 != 0) {
            if (tires[1].unk08 == tires[0].unk08) {
                tires[1].unk0 = 1;
            }
        } else if (tires[2].unk0 != 0) {
            if (tires[1].unk08 == tires[2].unk08) {
                tires[1].unk0 = 1;
            }
        } else if (tires[3].unk0 != 0) {
            if (tires[1].unk08 == tires[3].unk08) {
                tires[1].unk0 = 1;
            }
        }
    }
    if (tires[2].unk6 != 0) {
        if (tires[0].unk0 != 0) {
            if (tires[2].unk08 == tires[0].unk08) {
                tires[2].unk0 = 1;
            }
        } else if (tires[1].unk0 != 0) {
            if (tires[2].unk08 == tires[1].unk08) {
                tires[2].unk0 = 1;
            }
        } else if (tires[3].unk0 != 0) {
            if (tires[2].unk08 == tires[3].unk08) {
                tires[2].unk0 = 1;
            }
        }
    }
    if (tires[3].unk6 != 0) {
        if (tires[0].unk0 != 0) {
            if (tires[3].unk08 == tires[0].unk08) {
                tires[3].unk0 = 1;
            }
        } else if (tires[1].unk0 != 0) {
            if (tires[3].unk08 == tires[1].unk08) {
                tires[3].unk0 = 1;
            }
        } else if (tires[2].unk0 != 0) {
            if (tires[3].unk08 == tires[2].unk08) {
                tires[3].unk0 = 1;
            }
        }
    }
    if (tires[0].unk0 != 0 && numCheckBridges < tires[0].unk08) {
        if (tires[1].unk0 == 0) {
            GetBridgeGroundHeight(&tires[1], tires[0].unk08, h);
        }
        if (tires[2].unk0 == 0) {
            GetBridgeGroundHeight(&tires[2], tires[0].unk08, h);
        }
        if (tires[3].unk0 == 0) {
            GetBridgeGroundHeight(&tires[3], tires[0].unk08, h);
        }
    }
    if (tires[1].unk0 != 0 && numCheckBridges < tires[1].unk08) {
        if (tires[0].unk0 == 0) {
            GetBridgeGroundHeight(&tires[0], tires[1].unk08, h);
        }
        if (tires[2].unk0 == 0) {
            GetBridgeGroundHeight(&tires[2], tires[1].unk08, h);
        }
        if (tires[3].unk0 == 0) {
            GetBridgeGroundHeight(&tires[3], tires[1].unk08, h);
        }
    }
    if (tires[2].unk0 != 0 && numCheckBridges < tires[2].unk08) {
        if (tires[0].unk0 == 0) {
            GetBridgeGroundHeight(&tires[0], tires[2].unk08, h);
        }
        if (tires[1].unk0 == 0) {
            GetBridgeGroundHeight(&tires[1], tires[2].unk08, h);
        }
        if (tires[3].unk0 == 0) {
            GetBridgeGroundHeight(&tires[3], tires[2].unk08, h);
        }
    }
    if (tires[3].unk0 != 0 && numCheckBridges < tires[3].unk08) {
        if (tires[0].unk0 == 0) {
            GetBridgeGroundHeight(&tires[0], tires[3].unk08, h);
        }
        if (tires[1].unk0 == 0) {
            GetBridgeGroundHeight(&tires[1], tires[3].unk08, h);
        }
        if (tires[2].unk0 == 0) {
            GetBridgeGroundHeight(&tires[2], tires[3].unk08, h);
        }
    }
}

void CheckForBridge(CarTire* tire, u8 which, s32 h)
{
    s16 i;
    s16 n;

    if (which) {
        n = numBridges;
    } else {
        n = numCheckBridges;
    }
    for (i = 0; i < n; i++) {
        if (GetBridgeGroundHeight(tire, i, h)) {
            break;
        }
    }
    if (tire->unk0 == 0) {
        tire->unk22 = tire->unk24;
    }
}

u8 GetBridgeGroundHeight(CarTire* tire, s16 idx, s32 h)
{
    u8 r;
    s32 k;
    s32 deltaHeight;
    s32 extent;

    s16 i;
    s32 current;

    r = 0;
    if (idx >= 0 && numBridges >= idx) {
        if (tire->unk38 >= bridges[idx].minX && tire->unk3C >= bridges[idx].minZ
            && bridges[idx].minX + bridges[idx].extentX >= tire->unk38
            && bridges[idx].minZ + bridges[idx].extentZ >= tire->unk3C) {
            tire->unk22 = tire->unk24;
            k = bridges[idx].rampDir;
            if (k == 0) {
                extent = bridges[idx].extentZ;
                deltaHeight = bridges[idx].y1 - bridges[idx].y0;
                tire->unk22
                    += bridges[idx].y0 + deltaHeight * (tire->unk3C - bridges[idx].minZ) / extent;
            } else if (k == 1) {
                extent = bridges[idx].extentZ;
                deltaHeight = bridges[idx].y0 - bridges[idx].y1;
                tire->unk22 += bridges[idx].y1
                    + deltaHeight * (bridges[idx].minZ + extent - tire->unk3C) / extent;
            } else if (k == 2) {
                extent = bridges[idx].extentX;
                deltaHeight = bridges[idx].y1 - bridges[idx].y0;
                tire->unk22
                    += bridges[idx].y0 + deltaHeight * (tire->unk38 - bridges[idx].minX) / extent;
            } else {
                extent = bridges[idx].extentX;
                deltaHeight = bridges[idx].y0 - bridges[idx].y1;
                tire->unk22 += bridges[idx].y1
                    + deltaHeight * (bridges[idx].minX + extent - tire->unk38) / extent;
            }
            if (h < (s16)tire->unk22 - tire->unk40) {
                tire->unk6 = 1;
                tire->unk08 = idx;
                tire->unk22 = tire->unk24;
                r = 0;
            } else {
                tire->unk08 = idx;
                tire->unk6 = 0;
                tire->unk0E = bridges[idx].unk08;
                if (tire->unk0 == 0) {
                    if ((s16)tire->unk22 < (s16)tire->unk24) {
                        tire->unk3 = 1;
                    }
                    tire->unk26 = 0;
                }
                tire->unk0 = 1;
                r = 1;
            }
        } else {
            current = tire->unk08;
            if (current == idx) {
                if (tire->unk0 != 0) {
                    if (tire->unk3 == 0) {
                        tire->unk3 = 1;
                        tire->unk22 = tire->unk24;
                        tire->unk12 = 1;
                        if (tire->unk26 < 2) {
                            tire->unk14 = bridges[current].unk0A;
                        }
                    }
                    for (i = 0; i < 4; i++) {
                        if (bridges[idx].obj[i] == -1)
                            break;
                        r = GetBridgeGroundHeight(tire, bridges[idx].obj[i], h);
                    }
                } else if (tire->unk6 != 0) {
                    tire->unk6 = 0;
                }
            }
        }
    }
    return r;
}

void GetBridgeTireHeight(CarTire* tire, s32 h)
{
    s32 d;

    if (tire->unk3) {
        if ((s16)tire->unk12 > 0) {
            tire->unk28 = ~(GetFieldsLastFrame() * 2) * ((s16)tire->unk12 * (s16)tire->unk12);
        } else {
            tire->unk28 = 0;
        }
        if (tire->unk28 < -4096 - ((s16)tire->unk14 * 32)) {
            tire->unk28 = -4096 - ((s16)tire->unk14 * 32);
        } else {
            tire->unk12++;
        }
        d = tire->unk28;
        tire->unk16 = tire->unk16 + (tire->unk14 + d / 32);
        if ((s16)tire->unk24 + tire->unk16 <= (s16)tire->unk22) {
            tire->unk3 = 0;
            if (GetBridgeGroundHeight(tire, tire->unk08, h) == 0) {
                tire->unk0 = 0;
                tire->unk3 = 0;
                tire->unk16 = 0;
                tire->unk0E = 0;
                CheckForBridge(tire, 0, h);
                if (tire->unk0 == 0) {
                    tire->unk22 = tire->unk24;
                    tire->unk16 = 0;
                    tire->unk14 = -1;
                }
            }
        }
        tire->unk26 = 0;
    } else {
        tire->unk14 = tire->unk22 - (tire->unk24 + tire->unk16);
        (tire->unk26)++;
        if (h < (s16)tire->unk14) {
            tire->unk14 = h;
        } else if ((s16)tire->unk14 < -h) {
            tire->unk14 = -h;
        }
        tire->unk16 = tire->unk22 - tire->unk24;
        tire->unk12 = 1;
        tire->unk3 = 0;
        if (GetBridgeGroundHeight(tire, tire->unk08, h) == 0) {
            tire->unk0 = 0;
            tire->unk3 = 0;
            tire->unk16 = 0;
        }
    }
}

void CheckSlickSpots(Car* car, u8 isPlayer)
{
    CarMotion* m;
    u8* flags;
    CarStats* st;
    s16 i;
    s16 n;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        flags = PLAYER_CAR(car)->flags;
        st = &PLAYER_CAR(car)->stats;
    } else {
        m = &AI_CAR(car)->motion;
        flags = AI_CAR(car)->flags;
        st = &AI_CAR(car)->stats;
    }
    if (flags[18]) {
        st->unk1C -= GetFieldsLastFrame();
        if (st->unk1C <= 0) {
            flags[13] = 0;
            flags[18] = 0;
        } else {
            flags[13] = 1;
        }
    } else {
        flags[13] = 0;
    }
    n = numSlickSpots;
    for (i = 0; i < n; i++) {
        if (m->pos.vx < slickSpots[i].x) {
            continue;
        }
        if (m->pos.vy < slickSpots[i].z) {
            continue;
        }
        if (slickSpots[i].x + slickSpots[i].w < m->pos.vx) {
            continue;
        }
        if (slickSpots[i].z + slickSpots[i].h < m->pos.vy) {
            continue;
        }
        flags[13] = 1;
        return;
    }
}

void CheckCurbs(Car* car, u8 isPlayer)
{
    CarTire* tires;
    u8* flags;
    CarCollision* collision;

    if (isPlayer) {
        tires = PLAYER_CAR(car)->tires;
        flags = PLAYER_CAR(car)->flags;
        collision = &PLAYER_CAR(car)->collision;
    } else {
        tires = AI_CAR(car)->tires;
        flags = AI_CAR(car)->flags;
        collision = &AI_CAR(car)->collision;
    }
    if (tires[0].unk3 == 0) {
        CheckForCurb(&tires[0], collision);
    }
    if (tires[1].unk3 == 0) {
        CheckForCurb(&tires[1], collision);
    }
    if (tires[2].unk3 == 0) {
        CheckForCurb(&tires[2], collision);
    }
    if (tires[3].unk3 == 0) {
        CheckForCurb(&tires[3], collision);
    }
    flags[14] = tires[0].unk2 || tires[1].unk2 || tires[2].unk2 || tires[3].unk2;
}

void CheckForCurb(CarTire* tire, CarCollision* collision)
{
    s16 i;
    u8 found;

    found = 0;
    for (i = 0; i < numCurbs; i++) {
        if (tire->unk38 < curbs[i].x) {
            continue;
        }
        if (tire->unk3C < curbs[i].z) {
            continue;
        }
        if (curbs[i].x + curbs[i].w < tire->unk38) {
            continue;
        }
        if (curbs[i].z + curbs[i].h < tire->unk3C) {
            continue;
        }
        if (tire->unk2 == 0) {
            collision->unk12 = 0x11;
        }
        tire->unk2 = 1;
        found = 1;
        tire->unk0C = i;
        break;
    }
    if (found) {
        tire->unk1A = curbs[tire->unk0C].t;
    } else {
        tire->unk1A = 0;
    }
    if (tire->unk2 != 0 && found == 0) {
        collision->unk12 = 0xD;
    }
    tire->unk2 = found;
}

void CheckPotHoles(Car* car, u8 isPlayer)
{
    CarTire* tires;
    u8* flags;
    CarCollision* collision;

    if (isPlayer) {
        tires = PLAYER_CAR(car)->tires;
        flags = PLAYER_CAR(car)->flags;
        collision = &PLAYER_CAR(car)->collision;
    } else {
        tires = AI_CAR(car)->tires;
        flags = AI_CAR(car)->flags;
        collision = &AI_CAR(car)->collision;
    }
    if (tires[0].unk3 == 0) {
        CheckForPotHole(&tires[0], collision);
    }
    if (tires[1].unk3 == 0) {
        CheckForPotHole(&tires[1], collision);
    }
    if (tires[2].unk3 == 0) {
        CheckForPotHole(&tires[2], collision);
    }
    if (tires[3].unk3 == 0) {
        CheckForPotHole(&tires[3], collision);
    }
    flags[12] = tires[0].unk1 || tires[1].unk1 || tires[2].unk1 || tires[3].unk1;
}

void CheckForPotHole(CarTire* tire, CarCollision* collision)
{
    s16 i;
    u8 found;

    found = 0;
    for (i = 0; i < numCheckPotHoles; i++) {
        if (tire->unk38 < potHoles[i].x) {
            continue;
        }
        if (tire->unk3C < potHoles[i].z) {
            continue;
        }
        if (potHoles[i].x + potHoles[i].a < tire->unk38) {
            continue;
        }
        if (potHoles[i].z + potHoles[i].b < tire->unk3C) {
            continue;
        }
        if (tire->unk1 == 0) {
            collision->unk12 = 0x1D;
        }
        tire->unk1 = 1;
        found = 1;
        tire->unk0A = i;
        break;
    }
    if (found) {
        if (potHoles[(s16)tire->unk0A].active) {
            tire->unk18 = potHoles[(s16)tire->unk0A].c;
        } else {
            tire->unk18 = (potHoles[(s16)tire->unk0A].c
                              * (((tire->unk38 - potHoles[(s16)tire->unk0A].x) % 3)
                                  + ((tire->unk3C - potHoles[(s16)tire->unk0A].z) % 3)))
                / 6;
        }
    } else {
        tire->unk18 = 0;
    }
    tire->unk1 = found;
}

void UpdateRollingCar(Car* car, u8 isPlayer)
{
    CarMotion* m;
    CarStats* st;
    CarTire* tires;
    u8* flags;
    s32 lim;
    s32 r;
    s32 w;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        st = &PLAYER_CAR(car)->stats;
        tires = PLAYER_CAR(car)->tires;
        flags = PLAYER_CAR(car)->flags;
    } else {
        m = &AI_CAR(car)->motion;
        st = &AI_CAR(car)->stats;
        tires = AI_CAR(car)->tires;
        flags = AI_CAR(car)->flags;
    }
    if (m->vel.vy < 32 * (2850 * GetFieldsLastFrame() / 100)) {
        lim = 32 * (2850 * GetFieldsLastFrame() / 100);
    } else {
        lim = m->vel.vy;
    }
    r = m->vel.vy * (91 * GetFieldsLastFrame()) / lim;
    if (r < 0) {
        if (-45 * GetFieldsLastFrame() < r) {
            r = -45 * GetFieldsLastFrame();
        }
    } else if (r < 45 * GetFieldsLastFrame()) {
        r = 45 * GetFieldsLastFrame();
    }
    if (flags[11] == 0) {
        if (arena_rand() & 1) {
            explodeCreateFragments(1, &m->pos);
        }
        if ((arena_rand() & 1) == 0) {
            do_smoke(&m->pos);
        }
    }
    if (m->rotDelta.vy > 0) {
        r += abs(rsin(m->rot.vy)) * r / 4096;
        m->rot.vy = m->rot.vy + r;
    } else {
        r += abs(rsin(m->rot.vy)) * r / 4096;
        m->rot.vy = m->rot.vy - r;
    }
    if (abs(m->vel.vy) < 32 * (950 * GetFieldsLastFrame() / 100)) {
        if (abs(m->rot.vy) < abs(3 * r)) {
            flags[23] = 0;
            m->rotDelta.vy = 0;
            m->rot.vy = 0;
            SetNoCarDrift(m, flags);
        }
    } else {
        w = rsin(m->rot.vy) * st->unk74 / 4096;
        tires[0].unk40 += w;
        tires[1].unk40 += w;
        tires[2].unk40 += w;
        tires[3].unk40 += w;
        if (m->vel.vy != 0) {
            if (m->vel.vy > 0) {
                m->vel.vy = m->vel.vy - st->unkBC / 3;
            } else {
                m->vel.vy = m->vel.vy + st->unkBC / 3;
            }
        }
        m->rot2.vz = (s16)m->unk02;
        flags[1] = 1;
        m->rot.vz = SmoothAngleValue(m->rot.vz, m->unk04, 95);
    }
}

void UpdateTirePositions(Car* car, u8 isPlayer)
{
    CarMotion* m;
    CarStats* st;
    CarTire* tires;
    u8* flags;
    CarBounce* bounce;
    AICar* ai;
    s32 uaIndex;
    s32 spd;
    s32 hi;
    s32 lo;
    s32 lim;
    s32 d0;
    s32 d1;
    s32 t;
    s32 p;
    s32 ang;
    s32 ang2;
    s32 roll;
    s32 width;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        st = &PLAYER_CAR(car)->stats;
        tires = PLAYER_CAR(car)->tires;
        flags = PLAYER_CAR(car)->flags;
        uaIndex = PLAYER_CAR(car)->uaIndex;
        bounce = &PLAYER_CAR(car)->bounce;
        spd = 0;
        ai = 0;
    } else {
        m = &AI_CAR(car)->motion;
        st = &AI_CAR(car)->stats;
        tires = AI_CAR(car)->tires;
        flags = AI_CAR(car)->flags;
        bounce = &AI_CAR(car)->bounce;
        spd = AI_CAR(car)->unk38;
        ai = AI_CAR(car);
        uaIndex = AI_CAR(car)->uaIndex;
    }
    tires[0].unk40 = (s16)tires[0].unk24 + tires[0].unk16 + tires[0].unk18 + (s16)tires[0].unk1C
        + (s16)tires[0].unk1E + tires[0].unk20;
    if (tires[0].unk0 == 0) {
        tires[0].unk40 += tires[0].unk1A;
    }
    tires[1].unk40 = (s16)tires[1].unk24 + tires[1].unk16 + tires[1].unk18 + (s16)tires[1].unk1C
        + (s16)tires[1].unk1E + tires[1].unk20;
    if (tires[1].unk0 == 0) {
        tires[1].unk40 += tires[1].unk1A;
    }
    tires[2].unk40 = (s16)tires[2].unk24 + tires[2].unk16 + tires[2].unk18 + (s16)tires[2].unk1C
        + (s16)tires[2].unk1E + tires[2].unk20;
    if (tires[2].unk0 == 0) {
        tires[2].unk40 += tires[2].unk1A;
    }
    tires[3].unk40 = (s16)tires[3].unk24 + tires[3].unk16 + tires[3].unk18 + (s16)tires[3].unk1C
        + (s16)tires[3].unk1E + tires[3].unk20;
    if (tires[3].unk0 == 0) {
        tires[3].unk40 += tires[3].unk1A;
    }
    if (tires[0].unk40 < tires[1].unk40) {
        hi = tires[1].unk40;
    } else {
        hi = tires[0].unk40;
    }
    p = tires[2].unk40 < tires[3].unk40 ? tires[3].unk40 : tires[2].unk40;
    lo = p;
    if (hi >= lo) {
        lo = hi;
    }
    lim = 3 * st->unk78 / 4;
    lim = lo - lim;
    if (tires[0].unk40 < lim) {
        tires[0].unk40 = lim;
    }
    if (tires[1].unk40 < lim) {
        tires[1].unk40 = lim;
    }
    if (tires[2].unk40 < lim) {
        tires[2].unk40 = lim;
    }
    if (tires[3].unk40 < lim) {
        tires[3].unk40 = lim;
    }
    roll = ((tires[1].unk40 - tires[0].unk40) + (tires[3].unk40 - tires[2].unk40)) / 2;
    if (flags[23] != 0) {
        UpdateRollingCar(car, isPlayer);
    } else if (flags[0] == 0) {
        m->rot.vy = ratan2(roll, st->unk74);
        if (m->rot.vy >= 854) {
            m->rot.vy = 853;
        } else if (m->rot.vy < -853) {
            m->rot.vy = -853;
        }
        d0 = 0;
        if (tires[0].unk0 != tires[1].unk0 && tires[2].unk0 != tires[3].unk0) {
            d0 = (s16)tires[0].unk14 / GetFieldsLastFrame();
            d1 = (s16)tires[1].unk14 / GetFieldsLastFrame();
        } else {
            d1 = 0;
        }
        if (d0 + st->unk5C < d1 && m->vel.vy > st->unkC0) {
            flags[23] = 1;
            m->unk02 = m->rot2.vz;
            m->rotDelta.vy = 1;
            m->unk04 = m->rot2.vz + 1024;
        } else if (d1 + st->unk5C < d0 && st->unkA8 / 2 < m->vel.vy) {
            flags[23] = 1;
            m->unk02 = m->rot2.vz;
            m->rotDelta.vy = -1;
            m->unk04 = m->rot2.vz - 1024;
        }
    }
    if (flags[0] != 0 && flags[10] == 0) {
        m->rot.vx = SmoothAngleValue(m->rot.vx, 0, 98);
    } else {
        m->rot.vx = -ratan2(
            ((tires[0].unk40 - tires[2].unk40) + (tires[1].unk40 - tires[3].unk40)) / 2, st->unk78);
    }
    if (m->rot.vx >= 513) {
        m->rot.vx = 512;
    } else if (m->rot.vx < -512) {
        m->rot.vx = -512;
    }
    if (flags[10] == 0) {
        m->rot.vx += bounce->unk30;
        m->rot.vy += bounce->unk34;
    }
    m->pos.vz = (tires[0].unk40 + tires[1].unk40 + tires[2].unk40 + tires[3].unk40) / 4;
    width = st->unk74;
    ang = abs(m->rot.vy);
    t = width / 2 + 8;
    BoundAngle(&ang);
    if (ang < 1024) {
        hi = t * ang / 1024;
        hi += st->unk6C - st->unk6C * ang / 1024;
    } else {
        p = t * (ang - 1024);
        ang -= 1024;
        hi = t - p / 1024;
        hi += st->unk70 * ang / 1024;
    }
    m->pos.vz += hi;
    if (shellGetCurrentLevel() == LEVEL6_ROOFTOPS && st->unk40 > 0 && m->pos.vz < 160) {
        if (isPlayer == 0 && ai != 0) {
            if (spd >= 1601) {
                ang2 = 2048 - (GetPlayerInfo(ai->unk0E)->motion.rot.vz - ai->unk30);
                BoundAngle(&ang2);
                if (abs(ang2) >= 513) {
                    spd = 8000;
                }
            }
        }
        if ((spd >= 6401 || uaIndex == UA_BOSS_CAR) && ai != 0) {
            if (ai->unk16F == 0) {
                ai->stats.unk40 -= 20;
                if (ai->stats.unk40 <= 0) {
                    ai->unk16F = 0;
                } else {
                    ai->unk16F = 1;
                }
            } else {
                PutAICarBackOnRoof(ai);
            }
        } else {
            st->unk40 = -1;
        }
    }
}

void CheckHealthStands(Car* car, u8 isPlayer)
{
    CarMotion* m;
    CarTire* tires;
    u8* flags;
    s8* sel;
    CarStats* st;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        tires = PLAYER_CAR(car)->tires;
        flags = PLAYER_CAR(car)->flags;
        sel = &PLAYER_CAR(car)->standId;
        st = &PLAYER_CAR(car)->stats;
    } else {
        m = &AI_CAR(car)->motion;
        tires = AI_CAR(car)->tires;
        flags = AI_CAR(car)->flags;
        sel = &AI_CAR(car)->unk04;
        st = &AI_CAR(car)->stats;
    }
    if (!isPlayer) {
        flags[22] = 0;
        tires[0].unk0E = 0;
        tires[1].unk0E = 0;
        tires[2].unk0E = 0;
        tires[3].unk0E = 0;
    }
    if (flags[22]) {
        if (health_stand_active(*sel) == 0) {
            st->unk16 -= GetFieldsLastFrame();
        } else {
            st->unk16 = 0x3C;
        }
        if ((s16)st->unk16 <= 0) {
            flags[22] = 0;
            tires[0].unk0E = 0;
            tires[1].unk0E = 0;
            tires[2].unk0E = 0;
            tires[3].unk0E = 0;
        }
        return;
    }
    if (st->unk40 <= 0) {
        return;
    }
    if (m->vel.vy > 0) {
        if (tires[2].unk0E != 0) {
            flags[22] = 1;
            *sel = tires[2].unk0E;
        } else if (tires[3].unk0E != 0) {
            flags[22] = 1;
            *sel = tires[3].unk0E;
        }
    } else {
        if (tires[0].unk0E != 0 && health_stand_active(tires[0].unk0E)) {
            flags[22] = 1;
            *sel = tires[0].unk0E;
        } else if (tires[1].unk0E != 0 && health_stand_active(tires[1].unk0E)) {
            flags[22] = 1;
            *sel = tires[1].unk0E;
        }
    }
    if (flags[22]) {
        st->healthCap = st->unk40 + st->unk50;
        st->healthRate = ((st->healthCap - st->unk50) * GetFieldsLastFrame()) / 180;
        if (st->healthRate <= 0) {
            st->healthRate = 1;
        }
        return;
    }
    if (!isPlayer) {
        return;
    }
    if (st->unk01 != 0) {
        return;
    }
    if (m->vel.vy != 0) {
        return;
    }
    switch (shellGetCurrentLevel()) {
    case LEVEL2_WAREHOUSE:
        if (m->pos.vx >= -12464) {
            if (m->pos.vx < -12399) {
                if (m->pos.vy >= -4360) {
                    if (m->pos.vy < -4239) {
                        st->unk01 = 1;
                        st->unk40 = st->unk44;
                    }
                }
            }
        }
        break;
    case LEVEL3_FREEWAY:
        if (m->pos.vx >= 16288) {
            if (m->pos.vx < 16337) {
                if (m->pos.vy >= -7600) {
                    if (m->pos.vy < -7439) {
                        st->unk01 = 1;
                        st->unk40 = st->unk44;
                    }
                }
            }
        }
        break;
    case LEVEL4_CITY_PARK:
        if (m->pos.vx >= 11840) {
            if (m->pos.vx < 12161) {
                if (m->pos.vy >= -4000) {
                    if (m->pos.vy < -3919) {
                        st->unk01 = 1;
                        st->unk40 = st->unk44;
                    }
                }
            }
        }
        break;
    case LEVEL5_CYBURBIA:
        if (m->pos.vx >= 22640) {
            if (m->pos.vx < 22673) {
                if (m->pos.vy >= 11632) {
                    if (m->pos.vy < 11793) {
                        st->unk01 = 1;
                        st->unk40 = st->unk44;
                    }
                }
            }
        }
        break;
    case LEVEL6_ROOFTOPS:
        if (m->pos.vx >= 8272) {
            if (m->pos.vx < 8345) {
                if (m->pos.vy >= 10072) {
                    if (m->pos.vy < 10145) {
                        st->unk01 = 1;
                        st->unk40 = st->unk44;
                    }
                }
            }
        }
        break;
    }
}

void InitLevel3TriggerPoints(void)
{
    s16 i;
    s16 j;

    numTriggerPoints = 50;
    for (i = 0; i < numTriggerPoints; i++) {
        tPoints[i].f1A = 1;
        tPoints[i].type = 0;
        tPoints[i].f08 = 0;
        for (j = 0; j < 4; j++) {
            tPoints[i].link[j] = 0;
            tPoints[i].dist[j] = 0x78;
        }
    }
    triggerPtStartPts.n = 5;
    for (i = 0; i < 15; i++) {
        triggerPtStartPts.start[i] = i;
        triggerPtStartPts.grp[i] = 0;
    }
    triggerPtGroups.count = 1;
    triggerPtGroups.pos[0].vx = -0x5DC0;
    triggerPtGroups.pos[0].vy = -0x2EE0;
    triggerPtGroups.pos[0].vz = 0;
    triggerPtGroups.w[0] = 0xBB80;
    triggerPtGroups.d[0] = 0x5DC0;
    for (i = 0; i < triggerPtGroups.count; i++) {
        for (j = 0; j < triggerPtGroups.count; j++) {
            triggerPtGroups.a[i][j] = -1;
            triggerPtGroups.b[i][j] = 0;
            triggerPtGroups.minSpeed[i][j] = 0x3C;
        }
    }
    tPoints[0].vx = 0x2580;
    tPoints[0].vy = -0x2390;
    tPoints[0].f1A = 2;
    tPoints[0].link[0] = 6;
    tPoints[0].link[1] = 0x2F;
    tPoints[0].dist[0] = 0xA0;
    tPoints[0].dist[1] = 0xA0;
    tPoints[1].vx = -0x4110;
    tPoints[1].vy = 0x1630;
    tPoints[1].f1A = 2;
    tPoints[1].link[0] = 0x1D;
    tPoints[1].link[1] = 0x1E;
    tPoints[1].dist[0] = 0xA0;
    tPoints[1].dist[1] = 0xA0;
    tPoints[2].vx = 0x4110;
    tPoints[2].vy = 0x1630;
    tPoints[2].f1A = 2;
    tPoints[2].link[0] = 0x13;
    tPoints[2].link[1] = 0x14;
    tPoints[2].dist[0] = 0xA0;
    tPoints[2].dist[1] = 0xA0;
    tPoints[3].vx = -0x2580;
    tPoints[3].vy = -0x2390;
    tPoints[3].f1A = 2;
    tPoints[3].link[0] = 0x2C;
    tPoints[3].link[1] = 0x2D;
    tPoints[3].dist[0] = 0xA0;
    tPoints[3].dist[1] = 0xA0;
    tPoints[4].vx = 0;
    tPoints[4].vy = 0x2390;
    tPoints[4].f1A = 2;
    tPoints[4].link[0] = 0x19;
    tPoints[4].link[1] = 0x1A;
    tPoints[4].dist[0] = 0xA0;
    tPoints[4].dist[1] = 0xA0;
    tPoints[5].vx = 0x4110;
    tPoints[5].vy = -0xEE0;
    tPoints[5].f1A = 3;
    tPoints[5].link[0] = 9;
    tPoints[5].link[1] = 0xA;
    tPoints[5].link[2] = 0x30;
    tPoints[5].dist[2] = 0xA0;
    tPoints[6].vx = 0x3E20;
    tPoints[6].vy = -0x2390;
    tPoints[6].f1A = 2;
    tPoints[6].link[0] = 0;
    tPoints[6].link[1] = 7;
    tPoints[7].vx = 0x4020;
    tPoints[7].vy = -0x22D0;
    tPoints[7].f1A = 2;
    tPoints[7].link[0] = 6;
    tPoints[7].link[1] = 8;
    tPoints[8].vx = 0x4110;
    tPoints[8].vy = -0x20C0;
    tPoints[8].f1A = 2;
    tPoints[8].link[0] = 7;
    tPoints[8].link[1] = 9;
    tPoints[9].vx = 0x4110;
    tPoints[9].vy = -0x1770;
    tPoints[9].f1A = 2;
    tPoints[9].link[0] = 5;
    tPoints[9].link[1] = 8;
    tPoints[9].dist[0] = 0xA0;
    tPoints[9].dist[1] = 0xA0;
    tPoints[10].vx = 0x4230;
    tPoints[10].vy = -0xCD0;
    tPoints[10].f1A = 2;
    tPoints[10].link[0] = 5;
    tPoints[10].link[1] = 0xB;
    tPoints[10].dist[0] = 0x50;
    tPoints[10].dist[1] = 0x50;
    tPoints[11].vx = 0x43F8;
    tPoints[11].vy = -0xBE8;
    tPoints[11].f1A = 2;
    tPoints[11].link[0] = 0xA;
    tPoints[11].link[1] = 0xC;
    tPoints[11].dist[0] = 0x50;
    tPoints[11].dist[1] = 0x50;
    tPoints[12].vx = 0x45E0;
    tPoints[12].vy = -0xB30;
    tPoints[12].f1A = 2;
    tPoints[12].link[0] = 0xB;
    tPoints[12].link[1] = 0xD;
    tPoints[12].dist[0] = 0x50;
    tPoints[12].dist[1] = 0x50;
    tPoints[13].vx = 0x46A0;
    tPoints[13].vy = -0x920;
    tPoints[13].f1A = 2;
    tPoints[13].link[0] = 0xC;
    tPoints[13].link[1] = 0xE;
    tPoints[14].vx = 0x46A0;
    tPoints[14].vy = -0x488;
    tPoints[14].f1A = 2;
    tPoints[14].link[0] = 0xD;
    tPoints[14].link[1] = 0xF;
    tPoints[15].vx = 0x45E0;
    tPoints[15].vy = -0x2B0;
    tPoints[15].f1A = 2;
    tPoints[15].link[0] = 0xE;
    tPoints[15].link[1] = 0x10;
    tPoints[15].dist[0] = 0x50;
    tPoints[15].dist[1] = 0x50;
    tPoints[16].vx = 0x43D0;
    tPoints[16].vy = -0x1E0;
    tPoints[16].f1A = 2;
    tPoints[16].link[0] = 0xF;
    tPoints[16].link[1] = 0x11;
    tPoints[16].dist[0] = 0x50;
    tPoints[16].dist[1] = 0x50;
    tPoints[17].vx = 0x41E0;
    tPoints[17].vy = -0x110;
    tPoints[17].f1A = 2;
    tPoints[17].link[0] = 0x10;
    tPoints[17].link[1] = 0x12;
    tPoints[17].dist[0] = 0x50;
    tPoints[17].dist[1] = 0x50;
    tPoints[18].vx = 0x4110;
    tPoints[18].vy = 0xF0;
    tPoints[18].f1A = 3;
    tPoints[18].link[0] = 5;
    tPoints[18].link[1] = 0x11;
    tPoints[18].link[2] = 0x30;
    tPoints[19].vx = 0x4110;
    tPoints[19].vy = 0xB90;
    tPoints[19].f1A = 2;
    tPoints[19].link[0] = 0x12;
    tPoints[19].link[1] = 0x14;
    tPoints[19].dist[0] = 0xA0;
    tPoints[19].dist[1] = 0xA0;
    tPoints[20].vx = 0x4110;
    tPoints[20].vy = 0x20C0;
    tPoints[20].f1A = 2;
    tPoints[20].link[0] = 2;
    tPoints[20].link[1] = 0x13;
    tPoints[21].vx = 0x4020;
    tPoints[21].vy = 0x22D0;
    tPoints[21].f1A = 2;
    tPoints[21].link[0] = 2;
    tPoints[21].link[1] = 0x15;
    tPoints[22].vx = 0x3E20;
    tPoints[22].vy = 0x2390;
    tPoints[22].f1A = 2;
    tPoints[22].link[0] = 0x15;
    tPoints[22].link[1] = 0x17;
    tPoints[23].vx = 0x2580;
    tPoints[23].vy = 0x2390;
    tPoints[23].f1A = 2;
    tPoints[23].link[0] = 0x16;
    tPoints[23].link[1] = 0x18;
    tPoints[23].dist[0] = 0xA0;
    tPoints[23].dist[1] = 0xA0;
    tPoints[24].vx = 0x12C0;
    tPoints[24].vy = 0x2390;
    tPoints[24].f1A = 2;
    tPoints[24].link[0] = 0x17;
    tPoints[24].link[1] = 0x19;
    tPoints[24].dist[0] = 0xA0;
    tPoints[24].dist[1] = 0xA0;
    tPoints[25].vx = -0x12C0;
    tPoints[25].vy = 0x2390;
    tPoints[25].f1A = 2;
    tPoints[25].link[0] = 4;
    tPoints[25].link[1] = 0x18;
    tPoints[25].dist[0] = 0xA0;
    tPoints[25].dist[1] = 0xA0;
    tPoints[26].vx = -0x2580;
    tPoints[26].vy = 0x2390;
    tPoints[26].f1A = 2;
    tPoints[26].link[0] = 4;
    tPoints[26].link[1] = 0x1B;
    tPoints[26].dist[0] = 0xA0;
    tPoints[26].dist[1] = 0xA0;
    tPoints[27].vx = -0x3DF0;
    tPoints[27].vy = 0x2390;
    tPoints[27].f1A = 2;
    tPoints[27].link[0] = 0x1A;
    tPoints[27].link[1] = 0x1C;
    tPoints[28].vx = -0x4020;
    tPoints[28].vy = 0x22D0;
    tPoints[28].f1A = 2;
    tPoints[28].link[0] = 0x1B;
    tPoints[28].link[1] = 0x1D;
    tPoints[29].vx = -0x4110;
    tPoints[29].vy = 0x20C0;
    tPoints[29].f1A = 2;
    tPoints[29].link[0] = 1;
    tPoints[29].link[1] = 0x1C;
    tPoints[30].vx = -0x4110;
    tPoints[30].vy = 0xB90;
    tPoints[30].f1A = 2;
    tPoints[30].link[0] = 1;
    tPoints[30].link[1] = 0x1F;
    tPoints[30].dist[0] = 0xA0;
    tPoints[30].dist[1] = 0xA0;
    tPoints[31].vx = -0x4110;
    tPoints[31].vy = 0xF0;
    tPoints[31].f1A = 3;
    tPoints[31].link[0] = 0x1E;
    tPoints[31].link[1] = 0x20;
    tPoints[31].link[2] = 0x31;
    tPoints[32].vx = -0x41E0;
    tPoints[32].vy = -0x110;
    tPoints[32].f1A = 2;
    tPoints[32].link[0] = 0x1F;
    tPoints[32].link[1] = 0x21;
    tPoints[32].dist[0] = 0x50;
    tPoints[32].dist[1] = 0x50;
    tPoints[33].vx = -0x43D0;
    tPoints[33].vy = -0x1E0;
    tPoints[33].f1A = 2;
    tPoints[33].link[0] = 0x20;
    tPoints[33].link[1] = 0x22;
    tPoints[33].dist[0] = 0x50;
    tPoints[33].dist[1] = 0x50;
    tPoints[34].vx = -0x45E0;
    tPoints[34].vy = -0x2B0;
    tPoints[34].f1A = 2;
    tPoints[34].link[0] = 0x21;
    tPoints[34].link[1] = 0x23;
    tPoints[34].dist[0] = 0x50;
    tPoints[34].dist[1] = 0x50;
    tPoints[35].vx = -0x46A0;
    tPoints[35].vy = -0x488;
    tPoints[35].f1A = 2;
    tPoints[35].link[0] = 0x22;
    tPoints[35].link[1] = 0x24;
    tPoints[36].vx = -0x46A0;
    tPoints[36].vy = -0x920;
    tPoints[36].f1A = 2;
    tPoints[36].link[0] = 0x23;
    tPoints[36].link[1] = 0x25;
    tPoints[37].vx = -0x45E0;
    tPoints[37].vy = -0xB30;
    tPoints[37].f1A = 2;
    tPoints[37].link[0] = 0x24;
    tPoints[37].link[1] = 0x26;
    tPoints[38].vx = -0x43F8;
    tPoints[38].vy = -0xBE8;
    tPoints[38].f1A = 2;
    tPoints[38].link[0] = 0x25;
    tPoints[38].link[1] = 0x27;
    tPoints[39].vx = -0x4230;
    tPoints[39].vy = -0xCD0;
    tPoints[39].f1A = 2;
    tPoints[39].link[0] = 0x26;
    tPoints[39].link[1] = 0x28;
    tPoints[40].vx = -0x4110;
    tPoints[40].vy = -0xEE0;
    tPoints[40].f1A = 3;
    tPoints[40].link[0] = 0x27;
    tPoints[40].link[1] = 0x29;
    tPoints[40].link[2] = 0x31;
    tPoints[41].vx = -0x4110;
    tPoints[41].vy = -0x1770;
    tPoints[41].f1A = 2;
    tPoints[41].link[0] = 0x28;
    tPoints[41].link[1] = 0x2A;
    tPoints[41].dist[0] = 0xA0;
    tPoints[41].dist[1] = 0xA0;
    tPoints[42].vx = -0x4110;
    tPoints[42].vy = -0x20C0;
    tPoints[42].f1A = 2;
    tPoints[42].link[0] = 0x29;
    tPoints[42].link[1] = 0x2B;
    tPoints[43].vx = -0x4020;
    tPoints[43].vy = -0x22D0;
    tPoints[43].f1A = 2;
    tPoints[43].link[0] = 0x2A;
    tPoints[43].link[1] = 0x2C;
    tPoints[44].vx = -0x3E20;
    tPoints[44].vy = -0x2390;
    tPoints[44].f1A = 2;
    tPoints[44].link[0] = 3;
    tPoints[44].link[1] = 0x2B;
    tPoints[45].vx = -0x12C0;
    tPoints[45].vy = -0x2390;
    tPoints[45].f1A = 2;
    tPoints[45].link[0] = 3;
    tPoints[45].link[1] = 0x2E;
    tPoints[45].dist[0] = 0xA0;
    tPoints[45].dist[1] = 0xA0;
    tPoints[46].vx = 0;
    tPoints[46].vy = -0x2390;
    tPoints[46].f1A = 2;
    tPoints[46].link[0] = 0x2D;
    tPoints[46].link[1] = 0x2F;
    tPoints[46].dist[0] = 0xA0;
    tPoints[46].dist[1] = 0xA0;
    tPoints[47].vx = 0x12C0;
    tPoints[47].vy = -0x2390;
    tPoints[47].f1A = 2;
    tPoints[47].link[0] = 0x2E;
    tPoints[47].link[1] = 0;
    tPoints[47].dist[0] = 0xA0;
    tPoints[47].dist[1] = 0xA0;
    tPoints[48].vx = 0x40C0;
    tPoints[48].vy = -0x6A0;
    tPoints[48].f1A = 2;
    tPoints[48].link[0] = 5;
    tPoints[48].link[1] = 0x12;
    tPoints[48].dist[0] = 0xA0;
    tPoints[48].dist[1] = 0xA0;
    tPoints[49].vx = -0x4110;
    tPoints[49].vy = 0x780;
    tPoints[49].f1A = 2;
    tPoints[49].link[0] = 0x1F;
    tPoints[49].link[1] = 0x28;
    tPoints[49].dist[0] = 0xA0;
    tPoints[49].dist[1] = 0xA0;
}

void InitLevel3Bridges(void)
{
    s16 i;
    s16 j;

    numBridges = 0x12;
    numCheckBridges = 0xC;

    for (i = 0; i <= numBridges; i++) {
        bridges[i].unk08 = 0;
        bridges[i].unk0A = 10;
        for (j = 0; j < 4; j++) {
            bridges[i].obj[j] = -1;
        }
    }

    bridges[0].rampDir = 0;
    bridges[0].minX = -17280;
    bridges[0].minZ = 0x228;
    bridges[0].y0 = 0;
    bridges[0].y1 = 0x188;
    bridges[0].extentX = 0x4E0;
    bridges[0].extentZ = 0x438;
    bridges[0].obj[0] = 0x10;
    bridges[0].unk0A = 0x1E;

    bridges[1].rampDir = 1;
    bridges[1].minX = -17280;
    bridges[1].minZ = 0x960;
    bridges[1].y0 = 0x188;
    bridges[1].y1 = 0;
    bridges[1].extentX = 0x4E0;
    bridges[1].extentZ = 0x438;
    bridges[1].obj[0] = 0x10;
    bridges[1].unk0A = 0x1E;

    bridges[2].rampDir = 0;
    bridges[2].minX = 0x3EB0;
    bridges[2].minZ = 0x1F0;
    bridges[2].y0 = 0;
    bridges[2].y1 = 0xC0;
    bridges[2].extentX = 0x4E0;
    bridges[2].extentZ = 0x470;
    bridges[2].obj[0] = 0x11;
    bridges[2].unk0A = 0x1E;

    bridges[3].rampDir = 1;
    bridges[3].minX = 0x3EB0;
    bridges[3].minZ = 0x960;
    bridges[3].y0 = 0xC0;
    bridges[3].y1 = 0;
    bridges[3].extentX = 0x4E0;
    bridges[3].extentZ = 0x470;
    bridges[3].obj[0] = 0x11;
    bridges[3].unk0A = 0x1E;

    bridges[4].rampDir = 2;
    bridges[4].minX = 0x1230;
    bridges[4].minZ = 0x2000;
    bridges[4].y0 = 0;
    bridges[4].y1 = 0x18;
    bridges[4].extentX = 0x90;
    bridges[4].extentZ = 0x90;
    bridges[4].obj[0] = 0xC;

    bridges[5].rampDir = 3;
    bridges[5].minX = 0x1308;
    bridges[5].minZ = 0x2000;
    bridges[5].y0 = 0x18;
    bridges[5].y1 = 0;
    bridges[5].extentX = 0x90;
    bridges[5].extentZ = 0x90;
    bridges[5].obj[0] = 0xC;

    bridges[6].rampDir = 2;
    bridges[6].minX = -4944;
    bridges[6].minZ = 0x2000;
    bridges[6].y0 = 0;
    bridges[6].y1 = 0x18;
    bridges[6].extentX = 0x90;
    bridges[6].extentZ = 0x90;
    bridges[6].obj[0] = 0xD;

    bridges[7].rampDir = 3;
    bridges[7].minX = -4728;
    bridges[7].minZ = 0x2000;
    bridges[7].y0 = 0x18;
    bridges[7].y1 = 0;
    bridges[7].extentX = 0x90;
    bridges[7].extentZ = 0x90;
    bridges[7].obj[0] = 0xD;

    bridges[8].rampDir = 2;
    bridges[8].minX = 0x1230;
    bridges[8].minZ = -8336;
    bridges[8].y0 = 0;
    bridges[8].y1 = 0x18;
    bridges[8].extentX = 0x90;
    bridges[8].extentZ = 0x90;
    bridges[8].obj[0] = 0xE;

    bridges[9].rampDir = 3;
    bridges[9].minX = 0x1308;
    bridges[9].minZ = -8336;
    bridges[9].y0 = 0x18;
    bridges[9].y1 = 0;
    bridges[9].extentX = 0x90;
    bridges[9].extentZ = 0x90;
    bridges[9].obj[0] = 0xE;

    bridges[10].rampDir = 2;
    bridges[10].minX = -4944;
    bridges[10].minZ = -8336;
    bridges[10].y0 = 0;
    bridges[10].y1 = 0x18;
    bridges[10].extentX = 0x90;
    bridges[10].extentZ = 0x90;
    bridges[10].obj[0] = 0xF;

    bridges[11].rampDir = 3;
    bridges[11].minX = -4728;
    bridges[11].minZ = -8336;
    bridges[11].y0 = 0x18;
    bridges[11].y1 = 0;
    bridges[11].extentX = 0x90;
    bridges[11].extentZ = 0x90;
    bridges[11].obj[0] = 0xF;

    bridges[12].rampDir = 2;
    bridges[12].minX = 0x1278;
    bridges[12].minZ = 0x2000;
    bridges[12].y0 = 0x18;
    bridges[12].y1 = 0x18;
    bridges[12].extentX = 0x90;
    bridges[12].extentZ = 0x90;
    bridges[12].obj[0] = 4;
    bridges[12].obj[1] = 5;
    bridges[12].unk08 = 2;

    bridges[13].rampDir = 2;
    bridges[13].minX = -4872;
    bridges[13].minZ = 0x2000;
    bridges[13].y0 = 0x18;
    bridges[13].y1 = 0x18;
    bridges[13].extentX = 0x90;
    bridges[13].extentZ = 0x90;
    bridges[13].obj[0] = 6;
    bridges[13].obj[1] = 7;
    bridges[13].unk08 = 1;

    bridges[14].rampDir = 2;
    bridges[14].minX = 0x1278;
    bridges[14].minZ = -8336;
    bridges[14].y0 = 0x18;
    bridges[14].y1 = 0x18;
    bridges[14].extentX = 0x90;
    bridges[14].extentZ = 0x90;
    bridges[14].obj[0] = 8;
    bridges[14].obj[1] = 9;
    bridges[14].unk08 = 4;

    bridges[15].rampDir = 2;
    bridges[15].minX = -4872;
    bridges[15].minZ = -8336;
    bridges[15].y0 = 0x18;
    bridges[15].y1 = 0x18;
    bridges[15].extentX = 0x90;
    bridges[15].extentZ = 0x90;
    bridges[15].obj[0] = 0xA;
    bridges[15].obj[1] = 0xB;
    bridges[15].unk08 = 3;

    bridges[16].rampDir = 0;
    bridges[16].minX = -17280;
    bridges[16].minZ = 0x660;
    bridges[16].y0 = 0x188;
    bridges[16].y1 = 0x188;
    bridges[16].extentX = 0x4E0;
    bridges[16].extentZ = 0x300;
    bridges[16].obj[0] = 1;
    bridges[16].obj[1] = 0;

    bridges[17].rampDir = 0;
    bridges[17].minX = 0x3EB0;
    bridges[17].minZ = 0x660;
    bridges[17].y0 = 0xC0;
    bridges[17].y1 = 0xC0;
    bridges[17].extentX = 0x4E0;
    bridges[17].extentZ = 0x300;
    bridges[17].obj[0] = 3;
    bridges[17].obj[1] = 2;
}

void InitLevel3PotHoles(void)
{
    s16 i;
    s16 t;
    s32 n;

    numPotHoles = 2;
    numCheckPotHoles = 2;
    i = 0;
    do {
        potHoles[i].active = 0;
        t = i + 1;
        do {
        } while (0);
        n = numPotHoles;
        i = t;
    } while (t < n);

    potHoles[0].x = -16944;
    potHoles[0].z = 0x720;
    potHoles[0].a = 0x240;
    potHoles[0].b = 0x180;
    potHoles[0].c = -16;

    potHoles[1].x = 0x3FF0;
    potHoles[1].z = 0x720;
    potHoles[1].a = 0x240;
    potHoles[1].b = 0x180;
    potHoles[1].c = -16;
}

void InitLevel3Curbs(void)
{
    numCurbs = 0;
}

void InitLevel4TriggerPoints(void)
{
    s16 i;
    s16 j;

    numTriggerPoints = 42;
    for (i = 0; i < numTriggerPoints; i++) {
        tPoints[i].f1A = 1;
        tPoints[i].type = 0;
        tPoints[i].f08 = 0;
        for (j = 0; j < 4; j++) {
            tPoints[i].link[j] = 0;
            tPoints[i].dist[j] = 0xA0;
        }
    }
    triggerPtStartPts.n = 6;
    for (i = 0; i < 15; i++) {
        triggerPtStartPts.start[i] = i;
        triggerPtStartPts.grp[i] = 0;
    }
    triggerPtGroups.count = 1;
    triggerPtGroups.pos[0].vx = -0x1900;
    triggerPtGroups.pos[0].vy = -0x1F40;
    triggerPtGroups.pos[0].vz = 0;
    triggerPtGroups.w[0] = 0x5780;
    triggerPtGroups.d[0] = 0x3E80;
    for (i = 0; i < triggerPtGroups.count; i++) {
        for (j = 0; j < triggerPtGroups.count; j++) {
            triggerPtGroups.a[i][j] = -1;
            triggerPtGroups.b[i][j] = 0;
            triggerPtGroups.minSpeed[i][j] = 0x3C;
        }
    }
    tPoints[0].vx = 0x12C0;
    tPoints[0].vy = 0x960;
    tPoints[0].f1A = 4;
    tPoints[0].link[0] = 0xD;
    tPoints[0].link[1] = 0x22;
    tPoints[0].link[2] = 0x15;
    tPoints[0].link[3] = 0x1B;
    tPoints[1].vx = 0x2580;
    tPoints[1].vy = -0x960;
    tPoints[1].f1A = 3;
    tPoints[1].link[0] = 0x15;
    tPoints[1].link[1] = 0x18;
    tPoints[1].link[2] = 0x19;
    tPoints[2].vx = 0;
    tPoints[2].vy = -0x1B90;
    tPoints[2].f1A = 3;
    tPoints[2].link[0] = 7;
    tPoints[2].link[1] = 0x11;
    tPoints[2].link[2] = 0x12;
    tPoints[3].vx = -0x1230;
    tPoints[3].vy = 0x1B90;
    tPoints[3].f1A = 2;
    tPoints[3].link[0] = 0xA;
    tPoints[3].link[1] = 0xB;
    tPoints[4].vx = 0x37B0;
    tPoints[4].vy = 0x1B90;
    tPoints[4].f1A = 2;
    tPoints[4].link[0] = 0x1D;
    tPoints[4].link[1] = 0x1E;
    tPoints[5].vx = 0x37B0;
    tPoints[5].vy = -0x1B90;
    tPoints[5].f1A = 2;
    tPoints[5].link[0] = 0x17;
    tPoints[5].link[1] = 0x21;
    tPoints[6].vx = -0x1230;
    tPoints[6].vy = -0x90;
    tPoints[6].f1A = 3;
    tPoints[6].link[0] = 8;
    tPoints[6].link[1] = 9;
    tPoints[6].link[2] = 0xF;
    tPoints[7].vx = -0x1230;
    tPoints[7].vy = -0x1B90;
    tPoints[7].f1A = 2;
    tPoints[7].link[0] = 2;
    tPoints[7].link[1] = 8;
    tPoints[8].vx = -0x1230;
    tPoints[8].vy = -0x1350;
    tPoints[8].f1A = 3;
    tPoints[8].link[0] = 6;
    tPoints[8].link[1] = 7;
    tPoints[8].link[2] = 0x11;
    tPoints[9].vx = -0x1230;
    tPoints[9].vy = 0x528;
    tPoints[9].f1A = 2;
    tPoints[9].link[0] = 6;
    tPoints[9].link[1] = 0xA;
    tPoints[10].vx = -0x1230;
    tPoints[10].vy = 0x1230;
    tPoints[10].f1A = 3;
    tPoints[10].link[0] = 3;
    tPoints[10].link[1] = 9;
    tPoints[10].link[2] = 0xC;
    tPoints[11].vx = 0;
    tPoints[11].vy = 0x1B90;
    tPoints[11].f1A = 3;
    tPoints[11].link[0] = 3;
    tPoints[11].link[1] = 0xC;
    tPoints[11].link[2] = 0x14;
    tPoints[12].vx = 0;
    tPoints[12].vy = 0x1230;
    tPoints[12].f1A = 3;
    tPoints[12].link[0] = 0xA;
    tPoints[12].link[1] = 0xB;
    tPoints[12].link[2] = 0xD;
    tPoints[13].vx = 0;
    tPoints[13].vy = 0x960;
    tPoints[13].f1A = 3;
    tPoints[13].link[0] = 0;
    tPoints[13].link[1] = 0xC;
    tPoints[13].link[2] = 0xE;
    tPoints[14].vx = 0;
    tPoints[14].vy = 0x528;
    tPoints[14].f1A = 2;
    tPoints[14].link[0] = 0xD;
    tPoints[14].link[1] = 0xF;
    tPoints[15].vx = 0;
    tPoints[15].vy = -0x90;
    tPoints[15].f1A = 3;
    tPoints[15].link[0] = 6;
    tPoints[15].link[1] = 0xE;
    tPoints[15].link[2] = 0x10;
    tPoints[16].vx = 0;
    tPoints[16].vy = -0x960;
    tPoints[16].f1A = 3;
    tPoints[16].link[0] = 0xF;
    tPoints[16].link[1] = 0x11;
    tPoints[16].link[2] = 0x13;
    tPoints[17].vx = 0;
    tPoints[17].vy = -0x1350;
    tPoints[17].f1A = 3;
    tPoints[17].link[0] = 2;
    tPoints[17].link[1] = 8;
    tPoints[17].link[2] = 0x10;
    tPoints[18].vx = 0x8D0;
    tPoints[18].vy = -0x1B90;
    tPoints[18].f1A = 3;
    tPoints[18].link[0] = 2;
    tPoints[18].link[1] = 0x16;
    tPoints[18].link[2] = 0x25;
    tPoints[19].vx = 0x8D0;
    tPoints[19].vy = -0x960;
    tPoints[19].f1A = 3;
    tPoints[19].link[0] = 0x10;
    tPoints[19].link[1] = 0x15;
    tPoints[19].link[2] = 0x25;
    tPoints[20].vx = 0x12C0;
    tPoints[20].vy = 0x1B90;
    tPoints[20].f1A = 2;
    tPoints[20].link[0] = 0xB;
    tPoints[20].link[1] = 0x22;
    tPoints[21].vx = 0x12C0;
    tPoints[21].vy = -0x960;
    tPoints[21].f1A = 3;
    tPoints[21].link[0] = 0;
    tPoints[21].link[1] = 1;
    tPoints[21].link[2] = 0x13;
    tPoints[21].link[3] = 0x23;
    tPoints[22].vx = 0x12C0;
    tPoints[22].vy = -0x1B90;
    tPoints[22].f1A = 2;
    tPoints[22].link[0] = 0x12;
    tPoints[22].link[1] = 0x23;
    tPoints[23].vx = 0x2580;
    tPoints[23].vy = -0x1B90;
    tPoints[23].f1A = 2;
    tPoints[23].link[0] = 5;
    tPoints[23].link[1] = 0x18;
    tPoints[24].vx = 0x2580;
    tPoints[24].vy = -0x1350;
    tPoints[24].f1A = 3;
    tPoints[24].link[0] = 1;
    tPoints[24].link[1] = 0x17;
    tPoints[24].link[2] = 0x21;
    tPoints[25].vx = 0x2580;
    tPoints[25].vy = -0x90;
    tPoints[25].f1A = 3;
    tPoints[25].link[0] = 1;
    tPoints[25].link[1] = 0x1A;
    tPoints[25].link[2] = 0x20;
    tPoints[26].vx = 0x2580;
    tPoints[26].vy = 0x528;
    tPoints[26].f1A = 2;
    tPoints[26].link[0] = 0x19;
    tPoints[26].link[1] = 0x1B;
    tPoints[27].vx = 0x2580;
    tPoints[27].vy = 0x960;
    tPoints[27].f1A = 3;
    tPoints[27].link[0] = 0;
    tPoints[27].link[1] = 0x1A;
    tPoints[27].link[2] = 0x1C;
    tPoints[28].vx = 0x2580;
    tPoints[28].vy = 0x1230;
    tPoints[28].f1A = 3;
    tPoints[28].link[0] = 0x1B;
    tPoints[28].link[1] = 0x1D;
    tPoints[28].link[2] = 0x1E;
    tPoints[29].vx = 0x2580;
    tPoints[29].vy = 0x1B90;
    tPoints[29].f1A = 2;
    tPoints[29].link[0] = 4;
    tPoints[29].link[1] = 0x1C;
    tPoints[30].vx = 0x37B0;
    tPoints[30].vy = 0x1230;
    tPoints[30].f1A = 3;
    tPoints[30].link[0] = 0x1C;
    tPoints[30].link[1] = 0x1E;
    tPoints[30].link[2] = 0x20;
    tPoints[31].vx = 0x37B0;
    tPoints[31].vy = 0x528;
    tPoints[31].f1A = 2;
    tPoints[31].link[0] = 0x1E;
    tPoints[31].link[1] = 0x20;
    tPoints[32].vx = 0x37B0;
    tPoints[32].vy = -0x90;
    tPoints[32].f1A = 3;
    tPoints[32].link[0] = 0x19;
    tPoints[32].link[1] = 0x1F;
    tPoints[32].link[2] = 0x21;
    tPoints[33].vx = 0x37B0;
    tPoints[33].vy = -0x1350;
    tPoints[33].f1A = 3;
    tPoints[33].link[0] = 5;
    tPoints[33].link[1] = 0x18;
    tPoints[33].link[2] = 0x20;
    tPoints[34].vx = 0x12C0;
    tPoints[34].vy = 0xFC8;
    tPoints[34].f1A = 3;
    tPoints[34].link[0] = 0;
    tPoints[34].link[1] = 0x14;
    tPoints[34].link[2] = 0x1C;
    tPoints[35].vx = 0x12C0;
    tPoints[35].vy = -0x11D0;
    tPoints[35].f1A = 3;
    tPoints[35].link[0] = 0x15;
    tPoints[35].link[1] = 0x16;
    tPoints[35].link[2] = 0x24;
    tPoints[36].vx = 0x2580;
    tPoints[36].vy = -0xF78;
    tPoints[36].f1A = 3;
    tPoints[36].link[0] = 1;
    tPoints[36].link[1] = 0x18;
    tPoints[36].link[2] = 0x23;
    tPoints[37].vx = 0x960;
    tPoints[37].vy = -0x12C0;
    tPoints[37].f1A = 3;
    tPoints[37].link[0] = 0x12;
    tPoints[37].link[1] = 0x13;
    tPoints[37].link[2] = 0x23;
    tPoints[38].vx = 0x1D60;
    tPoints[38].vy = -0x1A90;
    tPoints[38].f1A = 1;
    tPoints[38].link[0] = 0x27;
    tPoints[39].vx = 0x1B80;
    tPoints[39].vy = -0x1428;
    tPoints[39].f1A = 2;
    tPoints[39].link[0] = 0x23;
    tPoints[39].link[1] = 0x24;
    tPoints[40].vx = 0x1C20;
    tPoints[40].vy = 0x1360;
    tPoints[40].f1A = 2;
    tPoints[40].link[0] = 0x1C;
    tPoints[40].link[1] = 0x22;
    tPoints[41].vx = 0x1CC0;
    tPoints[41].vy = 0x1A68;
    tPoints[41].f1A = 1;
    tPoints[41].link[0] = 0x28;
}

void InitLevel4Bridges(void)
{
    s16 i;
    s16 j;

    numBridges = 0x36;
    numCheckBridges = 0x2A;

    for (i = 0; i <= numBridges; i++) {
        bridges[i].unk08 = 0;
        bridges[i].unk0A = 10;
        for (j = 0; j < 4; j++) {
            bridges[i].obj[j] = -1;
        }
    }

    bridges[0].rampDir = 2;
    bridges[0].minX = 0x1620;
    bridges[0].minZ = -2688;
    bridges[0].y0 = 0;
    bridges[0].y1 = 0x110;
    bridges[0].extentX = 0x320;
    bridges[0].extentZ = 0x240;
    bridges[0].obj[0] = 2;

    bridges[1].rampDir = 3;
    bridges[1].minX = 0x1F00;
    bridges[1].minZ = -2688;
    bridges[1].y0 = 0x110;
    bridges[1].y1 = 0;
    bridges[1].extentX = 0x320;
    bridges[1].extentZ = 0x240;
    bridges[1].obj[0] = 2;

    bridges[2].rampDir = 2;
    bridges[2].minX = 0x1940;
    bridges[2].minZ = -2688;
    bridges[2].y0 = 0x110;
    bridges[2].y1 = 0x110;
    bridges[2].extentX = 0x5C0;
    bridges[2].extentZ = 0x240;
    bridges[2].obj[0] = 0;
    bridges[2].obj[1] = 1;

    bridges[3].rampDir = 2;
    bridges[3].minX = 0x1620;
    bridges[3].minZ = 0x840;
    bridges[3].y0 = 0;
    bridges[3].y1 = 0x110;
    bridges[3].extentX = 0x320;
    bridges[3].extentZ = 0x240;
    bridges[3].obj[0] = 5;

    bridges[4].rampDir = 3;
    bridges[4].minX = 0x1F00;
    bridges[4].minZ = 0x840;
    bridges[4].y0 = 0x110;
    bridges[4].y1 = 0;
    bridges[4].extentX = 0x320;
    bridges[4].extentZ = 0x240;
    bridges[4].obj[0] = 5;

    bridges[5].rampDir = 2;
    bridges[5].minX = 0x1940;
    bridges[5].minZ = 0x840;
    bridges[5].y0 = 0x110;
    bridges[5].y1 = 0x110;
    bridges[5].extentX = 0x5C0;
    bridges[5].extentZ = 0x240;
    bridges[5].obj[0] = 3;
    bridges[5].obj[1] = 4;

    bridges[6].rampDir = 2;
    bridges[6].minX = 0x1C20;
    bridges[6].minZ = -7488;
    bridges[6].y0 = -80;
    bridges[6].y1 = -80;
    bridges[6].extentX = 0x320;
    bridges[6].extentZ = 0x300;
    bridges[6].obj[0] = 0x2D;
    bridges[6].unk0A = 0;

    bridges[7].rampDir = 3;
    bridges[7].minX = 0x1A90;
    bridges[7].minZ = -5800;
    bridges[7].y0 = 8;
    bridges[7].y1 = -80;
    bridges[7].extentX = 0xA0;
    bridges[7].extentZ = 0x898;
    bridges[7].obj[0] = 0x2A;
    bridges[7].obj[1] = 0x2D;
    bridges[7].obj[2] = 0x2E;
    bridges[7].unk0A = 0x28;

    bridges[8].rampDir = 3;
    bridges[8].minX = 0x1D10;
    bridges[8].minZ = -5800;
    bridges[8].y0 = -80;
    bridges[8].y1 = 8;
    bridges[8].extentX = 0xA0;
    bridges[8].extentZ = 0x898;
    bridges[8].obj[0] = 0x2A;
    bridges[8].obj[1] = 0x2D;
    bridges[8].obj[2] = 0x2E;
    bridges[8].unk0A = 0x28;

    bridges[9].rampDir = 3;
    bridges[9].minX = 0x1B58;
    bridges[9].minZ = -2880;
    bridges[9].y0 = -80;
    bridges[9].y1 = -80;
    bridges[9].extentX = 0x320;
    bridges[9].extentZ = 0x3C0;
    bridges[9].obj[0] = 0x2E;
    bridges[9].obj[1] = 0x2F;
    bridges[9].unk0A = 0;

    bridges[10].rampDir = 2;
    bridges[10].minX = 0x1CE8;
    bridges[10].minZ = -1000;
    bridges[10].y0 = 8;
    bridges[10].y1 = -80;
    bridges[10].extentX = 0xA0;
    bridges[10].extentZ = 0x898;
    bridges[10].obj[0] = 0x2B;
    bridges[10].obj[1] = 0x2F;
    bridges[10].obj[2] = 0x30;
    bridges[10].unk0A = 0x28;

    bridges[11].rampDir = 3;
    bridges[11].minX = 0x1F68;
    bridges[11].minZ = -1000;
    bridges[11].y0 = -80;
    bridges[11].y1 = 8;
    bridges[11].extentX = 0xA0;
    bridges[11].extentZ = 0x898;
    bridges[11].obj[0] = 0x2B;
    bridges[11].obj[1] = 0x2F;
    bridges[11].obj[2] = 0x30;
    bridges[11].unk0A = 0x28;

    bridges[12].rampDir = 2;
    bridges[12].minX = 0x1C20;
    bridges[12].minZ = 0x780;
    bridges[12].y0 = -80;
    bridges[12].y1 = -80;
    bridges[12].extentX = 0x320;
    bridges[12].extentZ = 0x3C0;
    bridges[12].obj[0] = 0x30;
    bridges[12].obj[1] = 0x31;
    bridges[12].unk0A = 0;

    bridges[13].rampDir = 2;
    bridges[13].minX = 0x1A90;
    bridges[13].minZ = 0xED8;
    bridges[13].y0 = 8;
    bridges[13].y1 = -80;
    bridges[13].extentX = 0xA0;
    bridges[13].extentZ = 0x898;
    bridges[13].obj[0] = 0x2C;
    bridges[13].obj[1] = 0x31;
    bridges[13].obj[2] = 0x32;
    bridges[13].unk0A = 0x28;

    bridges[14].rampDir = 3;
    bridges[14].minX = 0x1D10;
    bridges[14].minZ = 0xED8;
    bridges[14].y0 = -80;
    bridges[14].y1 = 8;
    bridges[14].extentX = 0xA0;
    bridges[14].extentZ = 0x898;
    bridges[14].obj[0] = 0x2C;
    bridges[14].obj[1] = 0x31;
    bridges[14].obj[2] = 0x32;
    bridges[14].unk0A = 0x28;

    bridges[15].rampDir = 2;
    bridges[15].minX = 0x1B58;
    bridges[15].minZ = 0x1A40;
    bridges[15].y0 = -80;
    bridges[15].y1 = -80;
    bridges[15].extentX = 0x320;
    bridges[15].extentZ = 0x300;
    bridges[15].obj[0] = 0x32;
    bridges[15].unk0A = 0;

    bridges[16].rampDir = 2;
    bridges[16].minX = -1200;
    bridges[16].minZ = -456;
    bridges[16].y0 = 8;
    bridges[16].y1 = 0x20;
    bridges[16].extentX = 0x48;
    bridges[16].extentZ = 0x90;
    bridges[16].obj[0] = 0x33;

    bridges[17].rampDir = 3;
    bridges[17].minX = -984;
    bridges[17].minZ = -456;
    bridges[17].y0 = 0x20;
    bridges[17].y1 = 8;
    bridges[17].extentX = 0x48;
    bridges[17].extentZ = 0x90;
    bridges[17].obj[0] = 0x33;

    bridges[18].rampDir = 0;
    bridges[18].minX = 0x10F8;
    bridges[18].minZ = 0xE70;
    bridges[18].y0 = 8;
    bridges[18].y1 = 0x20;
    bridges[18].extentX = 0x90;
    bridges[18].extentZ = 0x48;
    bridges[18].obj[0] = 0x34;

    bridges[19].rampDir = 1;
    bridges[19].minX = 0x10F8;
    bridges[19].minZ = 0xF48;
    bridges[19].y0 = 0x20;
    bridges[19].y1 = 8;
    bridges[19].extentX = 0x90;
    bridges[19].extentZ = 0x48;
    bridges[19].obj[0] = 0x34;

    bridges[20].rampDir = 2;
    bridges[20].minX = 0x3210;
    bridges[20].minZ = -5256;
    bridges[20].y0 = 8;
    bridges[20].y1 = 0x20;
    bridges[20].extentX = 0x48;
    bridges[20].extentZ = 0x90;
    bridges[20].obj[0] = 0x35;

    bridges[21].rampDir = 3;
    bridges[21].minX = 0x32E8;
    bridges[21].minZ = -5256;
    bridges[21].y0 = 0x20;
    bridges[21].y1 = 8;
    bridges[21].extentX = 0x48;
    bridges[21].extentZ = 0x90;
    bridges[21].obj[0] = 0x35;

    bridges[22].rampDir = 0;
    bridges[22].minX = -9600;
    bridges[22].minZ = -1516;
    bridges[22].y0 = 0;
    bridges[22].y1 = 0x30;
    bridges[22].extentX = 0x60;
    bridges[22].extentZ = 0xB0;
    bridges[22].obj[0] = 0x17;
    bridges[22].unk0A = 0x1E;

    bridges[23].rampDir = 1;
    bridges[23].minX = -9600;
    bridges[23].minZ = -1340;
    bridges[23].y0 = 0x30;
    bridges[23].y1 = 0;
    bridges[23].extentX = 0x60;
    bridges[23].extentZ = 0xA8;
    bridges[23].obj[0] = 0x16;
    bridges[23].unk0A = 0x1E;

    bridges[24].rampDir = 0;
    bridges[24].minX = -9600;
    bridges[24].minZ = 0x194;
    bridges[24].y0 = 0;
    bridges[24].y1 = 0x30;
    bridges[24].extentX = 0x60;
    bridges[24].extentZ = 0xB0;
    bridges[24].obj[0] = 0x19;
    bridges[24].unk0A = 0x1E;

    bridges[25].rampDir = 1;
    bridges[25].minX = -9600;
    bridges[25].minZ = 0x244;
    bridges[25].y0 = 0x30;
    bridges[25].y1 = 0;
    bridges[25].extentX = 0x60;
    bridges[25].extentZ = 0xA8;
    bridges[25].obj[0] = 0x18;
    bridges[25].unk0A = 0x1E;

    bridges[26].rampDir = 2;
    bridges[26].minX = -3340;
    bridges[26].minZ = -288;
    bridges[26].y0 = 0;
    bridges[26].y1 = 0x30;
    bridges[26].extentX = 0xB0;
    bridges[26].extentZ = 0x60;
    bridges[26].obj[0] = 0x1B;
    bridges[26].unk0A = 0x1E;

    bridges[27].rampDir = 3;
    bridges[27].minX = -3164;
    bridges[27].minZ = -288;
    bridges[27].y0 = 0x30;
    bridges[27].y1 = 0;
    bridges[27].extentX = 0xA8;
    bridges[27].extentZ = 0x60;
    bridges[27].obj[0] = 0x1A;
    bridges[27].unk0A = 0x1E;

    bridges[28].rampDir = 2;
    bridges[28].minX = -2188;
    bridges[28].minZ = -192;
    bridges[28].y0 = 0;
    bridges[28].y1 = 0x30;
    bridges[28].extentX = 0xB0;
    bridges[28].extentZ = 0x60;
    bridges[28].obj[0] = 0x1D;
    bridges[28].unk0A = 0x1E;

    bridges[29].rampDir = 3;
    bridges[29].minX = -2012;
    bridges[29].minZ = -192;
    bridges[29].y0 = 0x30;
    bridges[29].y1 = 0;
    bridges[29].extentX = 0xA8;
    bridges[29].extentZ = 0x60;
    bridges[29].obj[0] = 0x1C;
    bridges[29].unk0A = 0x1E;

    bridges[30].rampDir = 0;
    bridges[30].minX = 0;
    bridges[30].minZ = -3628;
    bridges[30].y0 = 0;
    bridges[30].y1 = 0x30;
    bridges[30].extentX = 0x60;
    bridges[30].extentZ = 0xB0;
    bridges[30].obj[0] = 0x1F;
    bridges[30].unk0A = 0x1E;

    bridges[31].rampDir = 1;
    bridges[31].minX = 0;
    bridges[31].minZ = -3452;
    bridges[31].y0 = 0x30;
    bridges[31].y1 = 0;
    bridges[31].extentX = 0x60;
    bridges[31].extentZ = 0xA8;
    bridges[31].obj[0] = 0x1E;
    bridges[31].unk0A = 0x1E;

    bridges[32].rampDir = 2;
    bridges[32].minX = 0x374;
    bridges[32].minZ = -2400;
    bridges[32].y0 = 0;
    bridges[32].y1 = 0x30;
    bridges[32].extentX = 0xB0;
    bridges[32].extentZ = 0x60;
    bridges[32].obj[0] = 0x21;
    bridges[32].unk0A = 0x1E;

    bridges[33].rampDir = 3;
    bridges[33].minX = 0x424;
    bridges[33].minZ = -2400;
    bridges[33].y0 = 0x30;
    bridges[33].y1 = 0;
    bridges[33].extentX = 0xA8;
    bridges[33].extentZ = 0x60;
    bridges[33].obj[0] = 0x20;
    bridges[33].unk0A = 0x1E;

    bridges[34].rampDir = 2;
    bridges[34].minX = 0x4F4;
    bridges[34].minZ = 0x960;
    bridges[34].y0 = 0;
    bridges[34].y1 = 0x30;
    bridges[34].extentX = 0xB0;
    bridges[34].extentZ = 0x60;
    bridges[34].obj[0] = 0x23;
    bridges[34].unk0A = 0x1E;

    bridges[35].rampDir = 3;
    bridges[35].minX = 0x5A4;
    bridges[35].minZ = 0x960;
    bridges[35].y0 = 0x30;
    bridges[35].y1 = 0;
    bridges[35].extentX = 0xA8;
    bridges[35].extentZ = 0x60;
    bridges[35].obj[0] = 0x22;
    bridges[35].unk0A = 0x1E;

    bridges[36].rampDir = 2;
    bridges[36].minX = -8332;
    bridges[36].minZ = 0x8A0;
    bridges[36].y0 = 0;
    bridges[36].y1 = 0x30;
    bridges[36].extentX = 0xB0;
    bridges[36].extentZ = 0x60;
    bridges[36].obj[0] = 0x25;
    bridges[36].unk0A = 0x1E;

    bridges[37].rampDir = 3;
    bridges[37].minX = -8156;
    bridges[37].minZ = 0x8A0;
    bridges[37].y0 = 0x30;
    bridges[37].y1 = 0;
    bridges[37].extentX = 0xA8;
    bridges[37].extentZ = 0x60;
    bridges[37].obj[0] = 0x24;
    bridges[37].unk0A = 0x1E;

    bridges[38].rampDir = 2;
    bridges[38].minX = 0x1AB8;
    bridges[38].minZ = -7200;
    bridges[38].y0 = 0;
    bridges[38].y1 = 0x58;
    bridges[38].extentX = 0x140;
    bridges[38].extentZ = 0xA0;
    bridges[38].obj[0] = 0x27;
    bridges[38].unk0A = 0x1E;

    bridges[39].rampDir = 3;
    bridges[39].minX = 0x1F68;
    bridges[39].minZ = -7200;
    bridges[39].y0 = 0x58;
    bridges[39].y1 = 0;
    bridges[39].extentX = 0x140;
    bridges[39].extentZ = 0xA0;
    bridges[39].obj[0] = 0x26;
    bridges[39].unk0A = 0x1E;

    bridges[40].rampDir = 2;
    bridges[40].minX = 0x19F0;
    bridges[40].minZ = 0x1B80;
    bridges[40].y0 = 0;
    bridges[40].y1 = 0x58;
    bridges[40].extentX = 0x140;
    bridges[40].extentZ = 0xA0;
    bridges[40].obj[0] = 0x29;
    bridges[40].unk0A = 0x1E;

    bridges[41].rampDir = 3;
    bridges[41].minX = 0x1EA0;
    bridges[41].minZ = 0x1B80;
    bridges[41].y0 = 0x58;
    bridges[41].y1 = 0;
    bridges[41].extentX = 0x140;
    bridges[41].extentZ = 0xA0;
    bridges[41].obj[0] = 0x28;
    bridges[41].unk0A = 0x1E;

    bridges[42].rampDir = 2;
    bridges[42].minX = 0x1B30;
    bridges[42].minZ = -5800;
    bridges[42].y0 = -80;
    bridges[42].y1 = -80;
    bridges[42].extentX = 0x1E0;
    bridges[42].extentZ = 0x898;
    bridges[42].obj[0] = 7;
    bridges[42].obj[1] = 8;
    bridges[42].obj[2] = 0x2D;
    bridges[42].obj[3] = 0x2E;

    bridges[43].rampDir = 2;
    bridges[43].minX = 0x1D88;
    bridges[43].minZ = -1000;
    bridges[43].y0 = -80;
    bridges[43].y1 = -80;
    bridges[43].extentX = 0x1E0;
    bridges[43].extentZ = 0x898;
    bridges[43].obj[0] = 0xA;
    bridges[43].obj[1] = 0xB;
    bridges[43].obj[2] = 0x2F;
    bridges[43].obj[3] = 0x30;

    bridges[44].rampDir = 2;
    bridges[44].minX = 0x1B30;
    bridges[44].minZ = 0xED8;
    bridges[44].y0 = -80;
    bridges[44].y1 = -80;
    bridges[44].extentX = 0x1E0;
    bridges[44].extentZ = 0x898;
    bridges[44].obj[0] = 0xD;
    bridges[44].obj[1] = 0xE;
    bridges[44].obj[2] = 0x31;
    bridges[44].obj[3] = 0x32;

    bridges[45].rampDir = 2;
    bridges[45].minX = 0x1A90;
    bridges[45].minZ = -6720;
    bridges[45].y0 = -80;
    bridges[45].y1 = -80;
    bridges[45].extentX = 0x4B0;
    bridges[45].extentZ = 0x398;
    bridges[45].obj[0] = 6;
    bridges[45].obj[1] = 0x2A;
    bridges[45].unk0A = 0;

    bridges[46].rampDir = 2;
    bridges[46].minX = 0x1A90;
    bridges[46].minZ = -3600;
    bridges[46].y0 = -80;
    bridges[46].y1 = -80;
    bridges[46].extentX = 0x3E8;
    bridges[46].extentZ = 0x370;
    bridges[46].obj[0] = 9;
    bridges[46].obj[1] = 0x2A;
    bridges[46].unk0A = 0;

    bridges[47].rampDir = 2;
    bridges[47].minX = 0x1B58;
    bridges[47].minZ = -1920;
    bridges[47].y0 = -80;
    bridges[47].y1 = -80;
    bridges[47].extentX = 0x4B0;
    bridges[47].extentZ = 0x398;
    bridges[47].obj[0] = 9;
    bridges[47].obj[1] = 0x2B;
    bridges[47].unk0A = 0;

    bridges[48].rampDir = 2;
    bridges[48].minX = 0x1C20;
    bridges[48].minZ = 0x4B0;
    bridges[48].y0 = -80;
    bridges[48].y1 = -80;
    bridges[48].extentX = 0x3E8;
    bridges[48].extentZ = 0x2D0;
    bridges[48].obj[0] = 0xC;
    bridges[48].obj[1] = 0x2B;
    bridges[48].unk0A = 0;

    bridges[49].rampDir = 2;
    bridges[49].minX = 0x1A90;
    bridges[49].minZ = 0xB40;
    bridges[49].y0 = -80;
    bridges[49].y1 = -80;
    bridges[49].extentX = 0x4B0;
    bridges[49].extentZ = 0x398;
    bridges[49].obj[0] = 0xC;
    bridges[49].obj[1] = 0x2C;
    bridges[49].unk0A = 0;

    bridges[50].rampDir = 2;
    bridges[50].minX = 0x1A90;
    bridges[50].minZ = 0x1770;
    bridges[50].y0 = -80;
    bridges[50].y1 = -80;
    bridges[50].extentX = 0x3E8;
    bridges[50].extentZ = 0x4B0;
    bridges[50].obj[0] = 0xF;
    bridges[50].obj[1] = 0x2C;
    bridges[50].unk0A = 0;

    bridges[51].rampDir = 2;
    bridges[51].minX = -1128;
    bridges[51].minZ = -456;
    bridges[51].y0 = 0x20;
    bridges[51].y1 = 0x20;
    bridges[51].extentX = 0x90;
    bridges[51].extentZ = 0x90;
    bridges[51].obj[0] = 0x10;
    bridges[51].obj[1] = 0x11;
    bridges[51].unk08 = 1;

    bridges[52].rampDir = 0;
    bridges[52].minX = 0x10F8;
    bridges[52].minZ = 0xEB8;
    bridges[52].y0 = 0x20;
    bridges[52].y1 = 0x20;
    bridges[52].extentX = 0x90;
    bridges[52].extentZ = 0x90;
    bridges[52].obj[0] = 0x12;
    bridges[52].obj[1] = 0x13;
    bridges[52].unk08 = 2;

    bridges[53].rampDir = 2;
    bridges[53].minX = 0x3258;
    bridges[53].minZ = -5256;
    bridges[53].y0 = 0x20;
    bridges[53].y1 = 0x20;
    bridges[53].extentX = 0x90;
    bridges[53].extentZ = 0x90;
    bridges[53].obj[0] = 0x14;
    bridges[53].obj[1] = 0x15;
    bridges[53].unk08 = 3;
}

void InitLevel4Curbs(void)
{
    numCurbs = 16;
    curbs[0].x = -4512;
    curbs[0].z = -6912;
    curbs[0].w = 4224;
    curbs[0].h = 1824;
    curbs[0].t = 8;

    curbs[1].x = -4512;
    curbs[1].z = -4800;
    curbs[1].w = 4224;
    curbs[1].h = 2400;
    curbs[1].t = 8;

    curbs[2].x = -4512;
    curbs[2].z = -2112;
    curbs[2].w = 4224;
    curbs[2].h = 1824;
    curbs[2].t = 8;

    curbs[3].x = -4512;
    curbs[3].z = 0;
    curbs[3].w = 4224;
    curbs[3].h = 4800;
    curbs[3].t = 8;

    curbs[4].x = -4512;
    curbs[4].z = 5088;
    curbs[4].w = 4224;
    curbs[4].h = 1824;
    curbs[4].t = 8;

    curbs[5].x = 288;
    curbs[5].z = 2688;
    curbs[5].w = 4224;
    curbs[5].h = 4224;
    curbs[5].t = 8;

    curbs[6].x = 288;
    curbs[6].z = -2112;
    curbs[6].w = 4224;
    curbs[6].h = 4224;
    curbs[6].t = 8;

    curbs[7].x = 288;
    curbs[7].z = -6912;
    curbs[7].w = 4224;
    curbs[7].h = 4224;
    curbs[7].t = 8;

    curbs[8].x = 5088;
    curbs[8].z = -6912;
    curbs[8].w = 4224;
    curbs[8].h = 4224;
    curbs[8].t = 8;

    curbs[9].x = 5088;
    curbs[9].z = -2112;
    curbs[9].w = 4224;
    curbs[9].h = 4224;
    curbs[9].t = 8;

    curbs[10].x = 5088;
    curbs[10].z = 2688;
    curbs[10].w = 4224;
    curbs[10].h = 4224;
    curbs[10].t = 8;

    curbs[11].x = 9888;
    curbs[11].z = 5088;
    curbs[11].w = 4224;
    curbs[11].h = 1824;
    curbs[11].t = 8;

    curbs[12].x = 9888;
    curbs[12].z = 0;
    curbs[12].w = 4224;
    curbs[12].h = 4800;
    curbs[12].t = 8;

    curbs[13].x = 9888;
    curbs[13].z = -2112;
    curbs[13].w = 4224;
    curbs[13].h = 1824;
    curbs[13].t = 8;

    curbs[14].x = 9888;
    curbs[14].z = -4800;
    curbs[14].w = 4224;
    curbs[14].h = 2400;
    curbs[14].t = 8;

    curbs[15].x = 9888;
    curbs[15].z = -6912;
    curbs[15].w = 4224;
    curbs[15].h = 1824;
    curbs[15].t = 8;
}

void InitLevel4PotHoles(void)
{
    s16 i;
    s16 t;
    s32 n;

    numPotHoles = 1;
    numCheckPotHoles = 1;
    i = 0;
    do {
        potHoles[i].active = 0;
        t = i + 1;
        do {
        } while (0);
        n = numPotHoles;
        i = t;
    } while (t < n);

    potHoles[0].x = 0xC0;
    potHoles[0].z = 0xC0;
    potHoles[0].a = 0x1140;
    potHoles[0].b = 0x840;
    potHoles[0].c = 8;
    potHoles[0].active = 1;
}

void InitLevel4SlickSpots(void)
{
    numSlickSpots = 1;
    slickSpots[0].x = 0x16A8;
    slickSpots[0].z = -768;
    slickSpots[0].w = 0x3F8;
    slickSpots[0].h = 0x6F8;
}

void InitLevel5TriggerPoints(void)
{
    s16 i;
    s16 j;

    numTriggerPoints = 173;
    for (i = 0; i < numTriggerPoints; i++) {
        tPoints[i].f1A = 1;
        tPoints[i].type = 0;
        tPoints[i].f08 = 0;
        for (j = 0; j < 4; j++) {
            tPoints[i].link[j] = 0;
            tPoints[i].dist[j] = 0xA0;
        }
    }
    triggerPtStartPts.n = 12;
    for (i = 0; i < 15; i++) {
        triggerPtStartPts.start[i] = i;
        triggerPtStartPts.grp[i] = 0;
    }
    triggerPtGroups.count = 12;
    for (i = 0; i < triggerPtGroups.count; i++) {
        for (j = 0; j < triggerPtGroups.count; j++) {
            triggerPtGroups.a[i][j] = -1;
            triggerPtGroups.b[i][j] = 0;
            triggerPtGroups.minSpeed[i][j] = 0x3C;
        }
    }
    triggerPtGroups.pos[0].vx = 0x26C0;
    triggerPtGroups.pos[0].vy = 0x140;
    triggerPtGroups.pos[0].vz = 0;
    triggerPtGroups.w[0] = 0x35C0;
    triggerPtGroups.d[0] = 0x2300;
    triggerPtGroups.a[0][1] = 0x62;
    triggerPtGroups.a[0][2] = 0x61;
    triggerPtGroups.a[0][3] = 0x5A;
    triggerPtGroups.a[0][4] = 0x5F;
    triggerPtGroups.a[0][6] = 0x5A;
    triggerPtGroups.a[0][7] = 0x5A;
    triggerPtGroups.a[0][8] = 0x5A;
    triggerPtGroups.a[0][9] = 0x5C;
    triggerPtGroups.a[0][10] = 0x5C;
    triggerPtGroups.a[0][11] = 0x5C;
    triggerPtGroups.b[0][1] = 3;
    triggerPtGroups.b[0][2] = 3;
    triggerPtGroups.b[0][3] = 3;
    triggerPtGroups.b[0][4] = 4;
    triggerPtGroups.b[0][6] = 3;
    triggerPtGroups.b[0][7] = 3;
    triggerPtGroups.b[0][8] = 3;
    triggerPtGroups.b[0][9] = 3;
    triggerPtGroups.b[0][10] = 3;
    triggerPtGroups.b[0][11] = 3;
    triggerPtGroups.pos[1].vx = 0;
    triggerPtGroups.pos[1].vy = 0x1C20;
    triggerPtGroups.pos[1].vz = 0;
    triggerPtGroups.w[1] = 0x3840;
    triggerPtGroups.d[1] = 0x1C20;
    triggerPtGroups.a[1][0] = 0x31;
    triggerPtGroups.a[1][6] = 0x56;
    triggerPtGroups.a[1][7] = 0x56;
    triggerPtGroups.a[1][8] = 0x56;
    triggerPtGroups.b[1][0] = 3;
    triggerPtGroups.b[1][6] = 5;
    triggerPtGroups.b[1][7] = 5;
    triggerPtGroups.b[1][8] = 5;
    triggerPtGroups.pos[2].vx = 0x3840;
    triggerPtGroups.pos[2].vy = 0x1C20;
    triggerPtGroups.pos[2].vz = 0;
    triggerPtGroups.w[2] = 0x3840;
    triggerPtGroups.d[2] = 0x1C20;
    triggerPtGroups.a[2][0] = 0x3C;
    triggerPtGroups.a[2][6] = 0x58;
    triggerPtGroups.a[2][7] = 0x58;
    triggerPtGroups.a[2][8] = 0x58;
    triggerPtGroups.b[2][0] = 1;
    triggerPtGroups.b[2][6] = 5;
    triggerPtGroups.b[2][7] = 5;
    triggerPtGroups.b[2][8] = 5;
    triggerPtGroups.pos[3].vx = 0;
    triggerPtGroups.pos[3].vy = 0;
    triggerPtGroups.pos[3].vz = 0;
    triggerPtGroups.w[3] = 0x3840;
    triggerPtGroups.d[3] = 0x1C20;
    triggerPtGroups.a[3][0] = 0x27;
    triggerPtGroups.a[3][9] = 0x57;
    triggerPtGroups.a[3][10] = 0x57;
    triggerPtGroups.a[3][11] = 0x57;
    triggerPtGroups.b[3][0] = 0;
    triggerPtGroups.b[3][9] = 5;
    triggerPtGroups.b[3][10] = 5;
    triggerPtGroups.b[3][11] = 5;
    triggerPtGroups.pos[4].vx = 0x3840;
    triggerPtGroups.pos[4].vy = 0;
    triggerPtGroups.pos[4].vz = 0;
    triggerPtGroups.w[4] = 0x3840;
    triggerPtGroups.d[4] = 0x1C20;
    triggerPtGroups.a[4][0] = 0x47;
    triggerPtGroups.a[4][9] = 0x59;
    triggerPtGroups.a[4][10] = 0x59;
    triggerPtGroups.a[4][11] = 0x59;
    triggerPtGroups.b[4][0] = 0;
    triggerPtGroups.b[4][9] = 5;
    triggerPtGroups.b[4][10] = 5;
    triggerPtGroups.b[4][11] = 5;
    triggerPtGroups.pos[5].vx = -0x640;
    triggerPtGroups.pos[5].vy = -0x640;
    triggerPtGroups.pos[5].vz = 0;
    triggerPtGroups.w[5] = 0x7D00;
    triggerPtGroups.d[5] = 0x44C0;
    triggerPtGroups.pos[6].vx = -0xFA0;
    triggerPtGroups.pos[6].vy = 0x1D10;
    triggerPtGroups.pos[6].vz = 0;
    triggerPtGroups.w[6] = 0x3520;
    triggerPtGroups.d[6] = 0x2580;
    triggerPtGroups.a[6][0] = 0xA;
    triggerPtGroups.a[6][1] = 0x7E;
    triggerPtGroups.a[6][2] = 0x7E;
    triggerPtGroups.a[6][3] = 0x7E;
    triggerPtGroups.a[6][4] = 0x7E;
    triggerPtGroups.b[6][0] = 9;
    triggerPtGroups.b[6][1] = 5;
    triggerPtGroups.b[6][2] = 7;
    triggerPtGroups.b[6][3] = 5;
    triggerPtGroups.b[6][4] = 7;
    triggerPtGroups.pos[7].vx = 0x2580;
    triggerPtGroups.pos[7].vy = 0x3E80;
    triggerPtGroups.pos[7].vz = 0;
    triggerPtGroups.w[7] = 0x2580;
    triggerPtGroups.d[7] = 0x320;
    triggerPtGroups.a[7][0] = 0x7F;
    triggerPtGroups.a[7][1] = 0x80;
    triggerPtGroups.a[7][2] = 0x7F;
    triggerPtGroups.a[7][3] = 0x80;
    triggerPtGroups.a[7][4] = 0x7F;
    triggerPtGroups.b[7][0] = 5;
    triggerPtGroups.b[7][1] = 5;
    triggerPtGroups.b[7][2] = 5;
    triggerPtGroups.b[7][3] = 5;
    triggerPtGroups.b[7][4] = 5;
    triggerPtGroups.pos[8].vx = 0x4B00;
    triggerPtGroups.pos[8].vy = 0x1D10;
    triggerPtGroups.pos[8].vz = 0;
    triggerPtGroups.w[8] = 0x3520;
    triggerPtGroups.d[8] = 0x2580;
    triggerPtGroups.a[8][0] = 0x81;
    triggerPtGroups.a[8][1] = 0x81;
    triggerPtGroups.a[8][2] = 0x81;
    triggerPtGroups.a[8][3] = 0x81;
    triggerPtGroups.a[8][4] = 0x81;
    triggerPtGroups.b[8][0] = 7;
    triggerPtGroups.b[8][1] = 7;
    triggerPtGroups.b[8][2] = 5;
    triggerPtGroups.b[8][3] = 7;
    triggerPtGroups.b[8][4] = 7;
    triggerPtGroups.pos[9].vx = -0xFA0;
    triggerPtGroups.pos[9].vy = -0x960;
    triggerPtGroups.pos[9].vz = 0;
    triggerPtGroups.w[9] = 0x3520;
    triggerPtGroups.d[9] = 0x2670;
    triggerPtGroups.a[9][0] = 0x9A;
    triggerPtGroups.a[9][1] = 0xA9;
    triggerPtGroups.a[9][2] = 0xA9;
    triggerPtGroups.a[9][3] = 0x9A;
    triggerPtGroups.a[9][4] = 0x9A;
    triggerPtGroups.b[9][0] = 5;
    triggerPtGroups.b[9][1] = 6;
    triggerPtGroups.b[9][2] = 6;
    triggerPtGroups.b[9][3] = 5;
    triggerPtGroups.b[9][4] = 0xA;
    triggerPtGroups.pos[10].vx = 0x2580;
    triggerPtGroups.pos[10].vy = -0x960;
    triggerPtGroups.pos[10].vz = 0;
    triggerPtGroups.w[10] = 0x2580;
    triggerPtGroups.d[10] = 0x640;
    triggerPtGroups.a[10][0] = 0x99;
    triggerPtGroups.a[10][1] = 0x99;
    triggerPtGroups.a[10][2] = 0x98;
    triggerPtGroups.a[10][3] = 0x99;
    triggerPtGroups.a[10][4] = 0x98;
    triggerPtGroups.b[10][0] = 5;
    triggerPtGroups.b[10][1] = 9;
    triggerPtGroups.b[10][2] = 0xB;
    triggerPtGroups.b[10][3] = 5;
    triggerPtGroups.b[10][4] = 5;
    triggerPtGroups.pos[11].vx = 0x4B00;
    triggerPtGroups.pos[11].vy = -0x960;
    triggerPtGroups.pos[11].vz = 0;
    triggerPtGroups.w[11] = 0x3520;
    triggerPtGroups.d[11] = 0x2670;
    triggerPtGroups.a[11][0] = 0x97;
    triggerPtGroups.a[11][1] = 0x87;
    triggerPtGroups.a[11][2] = 0x87;
    triggerPtGroups.a[11][3] = 0x97;
    triggerPtGroups.a[11][4] = 0x97;
    triggerPtGroups.b[11][0] = 5;
    triggerPtGroups.b[11][1] = 8;
    triggerPtGroups.b[11][2] = 8;
    triggerPtGroups.b[11][3] = 0xA;
    triggerPtGroups.b[11][4] = 5;
    for (i = 13; i < 15; i++) {
        tPoints[i].type = 3;
    }
    for (i = 15; i < 22; i++) {
        tPoints[i].type = 1;
    }
    for (i = 22; i < 28; i++) {
        tPoints[i].type = 3;
    }
    for (i = 28; i < 35; i++) {
        tPoints[i].type = 1;
    }
    for (i = 35; i < 41; i++) {
        tPoints[i].type = 3;
    }
    for (i = 41; i < 47; i++) {
        tPoints[i].type = 1;
    }
    for (i = 47; i < 48; i++) {
        tPoints[i].type = 3;
    }
    for (i = 48; i < 52; i++) {
        tPoints[i].type = 1;
    }
    for (i = 52; i < 53; i++) {
        tPoints[i].type = 3;
    }
    for (i = 53; i < 55; i++) {
        tPoints[i].type = 4;
    }
    tPoints[55].type = 1;
    for (i = 56; i < 62; i++) {
        tPoints[i].type = 2;
    }
    for (i = 62; i < 65; i++) {
        tPoints[i].type = 4;
    }
    for (i = 65; i < 71; i++) {
        tPoints[i].type = 2;
    }
    for (i = 71; i < 77; i++) {
        tPoints[i].type = 4;
    }
    for (i = 77; i < 84; i++) {
        tPoints[i].type = 2;
    }
    for (i = 84; i < 86; i++) {
        tPoints[i].type = 4;
    }
    tPoints[86].type = 1;
    tPoints[87].type = 3;
    tPoints[88].type = 2;
    tPoints[89].type = 4;
    for (i = 90; i < 100; i++) {
        tPoints[i].type = 0;
    }
    for (i = 100; i < 124; i++) {
        tPoints[i].type = 5;
    }
    for (i = 124; i < 127; i++) {
        tPoints[i].type = 6;
    }
    for (i = 127; i < 129; i++) {
        tPoints[i].type = 7;
    }
    for (i = 129; i < 135; i++) {
        tPoints[i].type = 8;
    }
    for (i = 135; i < 152; i++) {
        tPoints[i].type = 11;
    }
    for (i = 152; i < 154; i++) {
        tPoints[i].type = 10;
    }
    for (i = 154; i < 170; i++) {
        tPoints[i].type = 9;
    }
    for (i = 170; i < 173; i++) {
        tPoints[i].type = 6;
    }
    Init2ndPartOfLevel5TriggerPts();
}

void Init2ndPartOfLevel5TriggerPts(void)
{
    tPoints[0].vx = 0x60;
    tPoints[0].vy = 0x60;
    tPoints[0].f1A = 2;
    tPoints[0].link[0] = 0xD;
    tPoints[0].link[1] = 0x18;
    tPoints[0].type = 3;
    tPoints[1].vx = 0x1C20;
    tPoints[1].vy = 0x12C0;
    tPoints[1].f1A = 3;
    tPoints[1].link[0] = 0x1B;
    tPoints[1].link[1] = 0x22;
    tPoints[1].link[2] = 0x27;
    tPoints[1].type = 3;
    tPoints[2].vx = 0x3E80;
    tPoints[2].vy = 0x758;
    tPoints[2].f1A = 3;
    tPoints[2].link[0] = 0x59;
    tPoints[2].link[1] = 0x5C;
    tPoints[2].link[2] = 0x5D;
    tPoints[2].type = 4;
    tPoints[3].vx = 0x5DC0;
    tPoints[3].vy = 0x2EE0;
    tPoints[3].f1A = 3;
    tPoints[3].link[0] = 0x44;
    tPoints[3].link[1] = 0x45;
    tPoints[3].link[2] = 0x4F;
    tPoints[3].type = 2;
    tPoints[4].vx = 0x12C0;
    tPoints[4].vy = 0x2EE0;
    tPoints[4].f1A = 3;
    tPoints[4].link[0] = 0x1D;
    tPoints[4].link[1] = 0x1E;
    tPoints[4].link[2] = 0x20;
    tPoints[4].type = 1;
    tPoints[5].vx = 0x7020;
    tPoints[5].vy = 0x60;
    tPoints[5].f1A = 2;
    tPoints[5].link[0] = 0x4A;
    tPoints[5].link[1] = 0x55;
    tPoints[5].type = 4;
    tPoints[6].vx = 0x3840;
    tPoints[6].vy = 0x2EE0;
    tPoints[6].f1A = 3;
    tPoints[6].link[0] = 0x30;
    tPoints[6].link[1] = 0x31;
    tPoints[6].link[2] = 0x38;
    tPoints[6].type = 1;
    tPoints[7].vx = 0x60;
    tPoints[7].vy = 0x2580;
    tPoints[7].f1A = 3;
    tPoints[7].link[0] = 0xF;
    tPoints[7].link[1] = 0x10;
    tPoints[7].link[2] = 0x14;
    tPoints[7].type = 1;
    tPoints[8].vx = 0x7020;
    tPoints[8].vy = 0x2580;
    tPoints[8].f1A = 3;
    tPoints[8].link[0] = 0x4E;
    tPoints[8].link[1] = 0x52;
    tPoints[8].link[2] = 0x53;
    tPoints[8].type = 2;
    tPoints[9].vx = 0x7950;
    tPoints[9].vy = 0x1C20;
    tPoints[9].f1A = 2;
    tPoints[9].link[0] = 0x86;
    tPoints[9].link[1] = 0x87;
    tPoints[9].type = 8;
    tPoints[10].vx = -0x8D0;
    tPoints[10].vy = 0x2BC0;
    tPoints[10].f1A = 2;
    tPoints[10].link[0] = 0xA9;
    tPoints[10].link[1] = 0xAA;
    tPoints[10].type = 6;
    tPoints[11].vx = 0x3840;
    tPoints[11].vy = 0x3FC0;
    tPoints[11].f1A = 2;
    tPoints[11].link[0] = 0x7F;
    tPoints[11].link[1] = 0x80;
    tPoints[11].type = 7;
    tPoints[12].vx = 0x3840;
    tPoints[12].vy = -0x780;
    tPoints[12].f1A = 2;
    tPoints[12].link[0] = 0x98;
    tPoints[12].link[1] = 0x99;
    tPoints[12].type = 0xA;
    tPoints[13].vx = 0x60;
    tPoints[13].vy = 0x960;
    tPoints[13].f1A = 2;
    tPoints[13].link[0] = 0;
    tPoints[13].link[1] = 0xE;
    tPoints[14].vx = 0x60;
    tPoints[14].vy = 0x12C0;
    tPoints[14].f1A = 3;
    tPoints[14].link[0] = 0xD;
    tPoints[14].link[1] = 0xF;
    tPoints[14].link[2] = 0x16;
    tPoints[15].vx = 0x60;
    tPoints[15].vy = 0x1C20;
    tPoints[15].f1A = 3;
    tPoints[15].link[0] = 7;
    tPoints[15].link[1] = 0xE;
    tPoints[15].link[2] = 0x15;
    tPoints[16].vx = 0x60;
    tPoints[16].vy = 0x2EE0;
    tPoints[16].f1A = 2;
    tPoints[16].link[0] = 7;
    tPoints[16].link[1] = 0x11;
    tPoints[17].vx = 0x60;
    tPoints[17].vy = 0x37E0;
    tPoints[17].f1A = 2;
    tPoints[17].link[0] = 0x10;
    tPoints[17].link[1] = 0x12;
    tPoints[18].vx = 0x960;
    tPoints[18].vy = 0x37E0;
    tPoints[18].f1A = 3;
    tPoints[18].link[0] = 0x11;
    tPoints[18].link[1] = 0x13;
    tPoints[18].link[2] = 0x1E;
    tPoints[19].vx = 0x960;
    tPoints[19].vy = 0x2EE0;
    tPoints[19].f1A = 2;
    tPoints[19].link[0] = 0x12;
    tPoints[19].link[1] = 0x14;
    tPoints[20].vx = 0x960;
    tPoints[20].vy = 0x2580;
    tPoints[20].f1A = 3;
    tPoints[20].link[0] = 7;
    tPoints[20].link[1] = 0x13;
    tPoints[20].link[2] = 0x1D;
    tPoints[21].vx = 0x960;
    tPoints[21].vy = 0x1C20;
    tPoints[21].f1A = 2;
    tPoints[21].link[0] = 0xF;
    tPoints[21].link[1] = 0x1C;
    tPoints[22].vx = 0x960;
    tPoints[22].vy = 0x12C0;
    tPoints[22].f1A = 2;
    tPoints[22].link[0] = 0xE;
    tPoints[22].link[1] = 0x1B;
    tPoints[23].vx = 0x960;
    tPoints[23].vy = 0x960;
    tPoints[23].f1A = 2;
    tPoints[23].link[0] = 0x16;
    tPoints[23].link[1] = 0x18;
    tPoints[24].vx = 0x960;
    tPoints[24].vy = 0x60;
    tPoints[24].f1A = 3;
    tPoints[24].link[0] = 0;
    tPoints[24].link[1] = 0x17;
    tPoints[24].link[2] = 0x19;
    tPoints[25].vx = 0x12C0;
    tPoints[25].vy = 0x60;
    tPoints[25].f1A = 3;
    tPoints[25].link[0] = 0x18;
    tPoints[25].link[1] = 0x1A;
    tPoints[25].link[2] = 0x24;
    tPoints[26].vx = 0x12C0;
    tPoints[26].vy = 0x960;
    tPoints[26].f1A = 3;
    tPoints[26].link[0] = 0x19;
    tPoints[26].link[1] = 0x1B;
    tPoints[26].link[2] = 0x23;
    tPoints[27].vx = 0x12C0;
    tPoints[27].vy = 0x12C0;
    tPoints[27].f1A = 4;
    tPoints[27].link[0] = 1;
    tPoints[27].link[1] = 0x16;
    tPoints[27].link[2] = 0x1A;
    tPoints[27].link[3] = 0x1C;
    tPoints[28].vx = 0x12C0;
    tPoints[28].vy = 0x1C20;
    tPoints[28].f1A = 3;
    tPoints[28].link[0] = 0x15;
    tPoints[28].link[1] = 0x1B;
    tPoints[28].link[2] = 0x1D;
    tPoints[29].vx = 0x12C0;
    tPoints[29].vy = 0x2580;
    tPoints[29].f1A = 4;
    tPoints[29].link[0] = 4;
    tPoints[29].link[1] = 0x14;
    tPoints[29].link[2] = 0x1C;
    tPoints[29].link[3] = 0x21;
    tPoints[30].vx = 0x12C0;
    tPoints[30].vy = 0x37E0;
    tPoints[30].f1A = 2;
    tPoints[30].link[0] = 4;
    tPoints[30].link[1] = 0x12;
    tPoints[31].vx = -0x1F40;
    tPoints[31].vy = -0x1F40;
    tPoints[31].f1A = 2;
    tPoints[31].link[0] = 0x1E;
    tPoints[31].link[1] = 0x2B;
    tPoints[32].vx = 0x1C20;
    tPoints[32].vy = 0x2EE0;
    tPoints[32].f1A = 2;
    tPoints[32].link[0] = 4;
    tPoints[32].link[1] = 0x2A;
    tPoints[33].vx = 0x1C20;
    tPoints[33].vy = 0x2580;
    tPoints[33].f1A = 2;
    tPoints[33].link[0] = 0x1D;
    tPoints[33].link[1] = 0x29;
    tPoints[33].link[2] = 0x22;
    tPoints[34].vx = 0x1C20;
    tPoints[34].vy = 0x1C20;
    tPoints[34].f1A = 2;
    tPoints[34].link[0] = 1;
    tPoints[34].link[1] = 0x21;
    tPoints[35].vx = 0x1C20;
    tPoints[35].vy = 0x960;
    tPoints[35].f1A = 2;
    tPoints[35].link[0] = 0x1A;
    tPoints[35].link[1] = 0x26;
    tPoints[36].vx = 0x1C20;
    tPoints[36].vy = 0x60;
    tPoints[36].f1A = 2;
    tPoints[36].link[0] = 0x19;
    tPoints[36].link[1] = 0x25;
    tPoints[37].vx = 0x2580;
    tPoints[37].vy = 0x60;
    tPoints[37].f1A = 3;
    tPoints[37].link[0] = 0x24;
    tPoints[37].link[1] = 0x26;
    tPoints[37].link[2] = 0x2F;
    tPoints[38].vx = 0x2580;
    tPoints[38].vy = 0x960;
    tPoints[38].f1A = 3;
    tPoints[38].link[0] = 0x23;
    tPoints[38].link[1] = 0x25;
    tPoints[38].link[2] = 0x27;
    tPoints[39].vx = 0x2580;
    tPoints[39].vy = 0x12C0;
    tPoints[39].f1A = 4;
    tPoints[39].link[0] = 1;
    tPoints[39].link[1] = 0x28;
    tPoints[39].link[2] = 0x57;
    tPoints[39].link[3] = 0x5A;
    tPoints[40].vx = 0x2580;
    tPoints[40].vy = 0x1C20;
    tPoints[40].f1A = 2;
    tPoints[40].link[0] = 0x27;
    tPoints[40].link[1] = 0x29;
    tPoints[41].vx = 0x2580;
    tPoints[41].vy = 0x2580;
    tPoints[41].f1A = 4;
    tPoints[41].link[0] = 0x21;
    tPoints[41].link[1] = 0x28;
    tPoints[41].link[2] = 0x2E;
    tPoints[41].link[3] = 0x56;
    tPoints[42].vx = 0x2580;
    tPoints[42].vy = 0x2EE0;
    tPoints[42].f1A = 3;
    tPoints[42].link[0] = 0x20;
    tPoints[42].link[1] = 0x29;
    tPoints[42].link[2] = 0x2B;
    tPoints[43].vx = 0x2580;
    tPoints[43].vy = 0x37E0;
    tPoints[43].f1A = 2;
    tPoints[43].link[0] = 0x2A;
    tPoints[43].link[1] = 0x2C;
    tPoints[44].vx = 0x2EE0;
    tPoints[44].vy = 0x37E0;
    tPoints[44].f1A = 3;
    tPoints[44].link[0] = 0x2B;
    tPoints[44].link[1] = 0x2D;
    tPoints[44].link[2] = 0x30;
    tPoints[45].vx = 0x2EE0;
    tPoints[45].vy = 0x2EE0;
    tPoints[45].f1A = 2;
    tPoints[45].link[0] = 0x2C;
    tPoints[45].link[1] = 0x2E;
    tPoints[46].vx = 0x2EE0;
    tPoints[46].vy = 0x2580;
    tPoints[46].f1A = 3;
    tPoints[46].link[0] = 0x29;
    tPoints[46].link[1] = 0x2D;
    tPoints[46].link[2] = 0x31;
    tPoints[47].vx = 0x2EE0;
    tPoints[47].vy = 0x60;
    tPoints[47].f1A = 2;
    tPoints[47].link[0] = 0x25;
    tPoints[47].link[1] = 0x34;
    tPoints[48].vx = 0x3840;
    tPoints[48].vy = 0x37E0;
    tPoints[48].f1A = 3;
    tPoints[48].link[0] = 6;
    tPoints[48].link[1] = 0x2C;
    tPoints[48].link[2] = 0x39;
    tPoints[49].vx = 0x3840;
    tPoints[49].vy = 0x2580;
    tPoints[49].f1A = 4;
    tPoints[49].link[0] = 6;
    tPoints[49].link[1] = 0x2E;
    tPoints[49].link[2] = 0x37;
    tPoints[49].link[3] = 0x62;
    tPoints[50].vx = -0x1F40;
    tPoints[50].vy = -0x1F40;
    tPoints[50].f1A = 2;
    tPoints[50].link[0] = 0x31;
    tPoints[50].link[1] = 0x33;
    tPoints[51].vx = -0x1F40;
    tPoints[51].vy = -0x1F40;
    tPoints[51].f1A = 2;
    tPoints[51].link[0] = 0x32;
    tPoints[51].link[1] = 0x36;
    tPoints[52].vx = 0x3840;
    tPoints[52].vy = 0x60;
    tPoints[52].f1A = 3;
    tPoints[52].link[0] = 0x2F;
    tPoints[52].link[1] = 0x35;
    tPoints[52].link[2] = 0x5C;
    tPoints[53].vx = 0x41A0;
    tPoints[53].vy = 0x60;
    tPoints[53].f1A = 2;
    tPoints[53].link[0] = 0x34;
    tPoints[53].link[1] = 0x3F;
    tPoints[54].vx = -0x1F40;
    tPoints[54].vy = -0x1F40;
    tPoints[54].f1A = 2;
    tPoints[54].link[0] = 2;
    tPoints[54].link[1] = 0x33;
    tPoints[55].vx = 0x41A0;
    tPoints[55].vy = 0x2580;
    tPoints[55].f1A = 2;
    tPoints[55].link[0] = 0x31;
    tPoints[55].link[1] = 0x3C;
    tPoints[56].vx = 0x41A0;
    tPoints[56].vy = 0x2EE0;
    tPoints[56].f1A = 2;
    tPoints[56].link[0] = 6;
    tPoints[56].link[1] = 0x3B;
    tPoints[57].vx = 0x41A0;
    tPoints[57].vy = 0x37E0;
    tPoints[57].f1A = 2;
    tPoints[57].link[0] = 0x30;
    tPoints[57].link[1] = 0x3A;
    tPoints[58].vx = 0x4B00;
    tPoints[58].vy = 0x37E0;
    tPoints[58].f1A = 3;
    tPoints[58].link[0] = 0x39;
    tPoints[58].link[1] = 0x3B;
    tPoints[58].link[2] = 0x43;
    tPoints[59].vx = 0x4B00;
    tPoints[59].vy = 0x2EE0;
    tPoints[59].f1A = 3;
    tPoints[59].link[0] = 0x38;
    tPoints[59].link[1] = 0x3A;
    tPoints[59].link[2] = 0x3C;
    tPoints[60].vx = 0x4B00;
    tPoints[60].vy = 0x2580;
    tPoints[60].f1A = 4;
    tPoints[60].link[0] = 0x37;
    tPoints[60].link[1] = 0x41;
    tPoints[60].link[2] = 0x58;
    tPoints[60].link[3] = 0x61;
    tPoints[61].vx = -0x1F40;
    tPoints[61].vy = -0x1F40;
    tPoints[61].f1A = 2;
    tPoints[61].link[0] = 2;
    tPoints[61].link[1] = 0x3C;
    tPoints[62].vx = -0x1F40;
    tPoints[62].vy = -0x1F40;
    tPoints[62].f1A = 2;
    tPoints[62].link[0] = 2;
    tPoints[62].link[1] = 0x3F;
    tPoints[63].vx = 0x4B00;
    tPoints[63].vy = 0x60;
    tPoints[63].f1A = 3;
    tPoints[63].link[0] = 0x35;
    tPoints[63].link[1] = 0x40;
    tPoints[63].link[2] = 0x59;
    tPoints[64].vx = 0x5460;
    tPoints[64].vy = 0x60;
    tPoints[64].f1A = 2;
    tPoints[64].link[0] = 0x3F;
    tPoints[64].link[1] = 0x49;
    tPoints[65].vx = 0x5460;
    tPoints[65].vy = 0x2580;
    tPoints[65].f1A = 3;
    tPoints[65].link[0] = 0x3C;
    tPoints[65].link[1] = 0x42;
    tPoints[65].link[2] = 0x45;
    tPoints[66].vx = 0x5460;
    tPoints[66].vy = 0x2EE0;
    tPoints[66].f1A = 2;
    tPoints[66].link[0] = 0x41;
    tPoints[66].link[1] = 0x43;
    tPoints[67].vx = 0x5460;
    tPoints[67].vy = 0x37E0;
    tPoints[67].f1A = 3;
    tPoints[67].link[0] = 0x3A;
    tPoints[67].link[1] = 0x42;
    tPoints[67].link[2] = 0x44;
    tPoints[68].vx = 0x5DC0;
    tPoints[68].vy = 0x37E0;
    tPoints[68].f1A = 3;
    tPoints[68].link[0] = 3;
    tPoints[68].link[1] = 0x43;
    tPoints[68].link[2] = 0x50;
    tPoints[69].vx = 0x5DC0;
    tPoints[69].vy = 0x2580;
    tPoints[69].f1A = 4;
    tPoints[69].link[0] = 3;
    tPoints[69].link[1] = 0x41;
    tPoints[69].link[2] = 0x46;
    tPoints[69].link[3] = 0x4E;
    tPoints[70].vx = 0x5DC0;
    tPoints[70].vy = 0x1C20;
    tPoints[70].f1A = 2;
    tPoints[70].link[0] = 0x45;
    tPoints[70].link[1] = 0x47;
    tPoints[71].vx = 0x5DC0;
    tPoints[71].vy = 0x12C0;
    tPoints[71].f1A = 4;
    tPoints[71].link[0] = 0x46;
    tPoints[71].link[1] = 0x48;
    tPoints[71].link[2] = 0x4C;
    tPoints[71].link[3] = 0x5F;
    tPoints[72].vx = 0x5DC0;
    tPoints[72].vy = 0x960;
    tPoints[72].f1A = 3;
    tPoints[72].link[0] = 0x47;
    tPoints[72].link[1] = 0x49;
    tPoints[72].link[2] = 0x4B;
    tPoints[73].vx = 0x5DC0;
    tPoints[73].vy = 0x60;
    tPoints[73].f1A = 3;
    tPoints[73].link[0] = 0x40;
    tPoints[73].link[1] = 0x48;
    tPoints[73].link[2] = 0x4A;
    tPoints[74].vx = 0x6720;
    tPoints[74].vy = 0x60;
    tPoints[74].f1A = 2;
    tPoints[74].link[0] = 5;
    tPoints[74].link[1] = 0x49;
    tPoints[75].vx = 0x6720;
    tPoints[75].vy = 0x960;
    tPoints[75].f1A = 2;
    tPoints[75].link[0] = 0x48;
    tPoints[75].link[1] = 0x55;
    tPoints[76].vx = 0x6720;
    tPoints[76].vy = 0x12C0;
    tPoints[76].f1A = 3;
    tPoints[76].link[0] = 0x47;
    tPoints[76].link[1] = 0x4D;
    tPoints[76].link[2] = 0x54;
    tPoints[77].vx = 0x6720;
    tPoints[77].vy = 0x1C20;
    tPoints[77].f1A = 2;
    tPoints[77].link[0] = 0x4C;
    tPoints[77].link[1] = 0x4E;
    tPoints[78].vx = 0x6720;
    tPoints[78].vy = 0x2580;
    tPoints[78].f1A = 3;
    tPoints[78].link[0] = 8;
    tPoints[78].link[1] = 0x45;
    tPoints[78].link[2] = 0x4D;
    tPoints[79].vx = 0x6720;
    tPoints[79].vy = 0x2EE0;
    tPoints[79].f1A = 2;
    tPoints[79].link[0] = 3;
    tPoints[79].link[1] = 0x52;
    tPoints[80].vx = 0x6720;
    tPoints[80].vy = 0x37E0;
    tPoints[80].f1A = 2;
    tPoints[80].link[0] = 0x44;
    tPoints[80].link[1] = 0x51;
    tPoints[81].vx = 0x7020;
    tPoints[81].vy = 0x37E0;
    tPoints[81].f1A = 2;
    tPoints[81].link[0] = 0x50;
    tPoints[81].link[1] = 0x52;
    tPoints[82].vx = 0x7020;
    tPoints[82].vy = 0x2EE0;
    tPoints[82].f1A = 3;
    tPoints[82].link[0] = 8;
    tPoints[82].link[1] = 0x4F;
    tPoints[82].link[2] = 0x51;
    tPoints[83].vx = 0x7020;
    tPoints[83].vy = 0x1C20;
    tPoints[83].f1A = 2;
    tPoints[83].link[0] = 8;
    tPoints[83].link[1] = 0x54;
    tPoints[84].vx = 0x7020;
    tPoints[84].vy = 0x12C0;
    tPoints[84].f1A = 3;
    tPoints[84].link[0] = 0x4C;
    tPoints[84].link[1] = 0x53;
    tPoints[84].link[2] = 0x55;
    tPoints[85].vx = 0x7020;
    tPoints[85].vy = 0x960;
    tPoints[85].f1A = 3;
    tPoints[85].link[0] = 5;
    tPoints[85].link[1] = 0x4B;
    tPoints[85].link[2] = 0x54;
    tPoints[86].vx = 0x2580;
    tPoints[86].vy = 0x2A30;
    tPoints[86].f1A = 4;
    tPoints[86].link[0] = 0x29;
    tPoints[86].link[1] = 0x2A;
    tPoints[86].link[2] = 0x64;
    tPoints[86].link[3] = 0x67;
    tPoints[87].vx = 0x2580;
    tPoints[87].vy = 0xE10;
    tPoints[87].f1A = 4;
    tPoints[87].link[0] = 0x26;
    tPoints[87].link[1] = 0x27;
    tPoints[87].link[2] = 0x70;
    tPoints[87].link[3] = 0x73;
    tPoints[88].vx = 0x4B00;
    tPoints[88].vy = 0x2A30;
    tPoints[88].f1A = 4;
    tPoints[88].link[0] = 0x3B;
    tPoints[88].link[1] = 0x3C;
    tPoints[88].link[2] = 0x6A;
    tPoints[88].link[3] = 0x6D;
    tPoints[89].vx = 0x4B00;
    tPoints[89].vy = 0x550;
    tPoints[89].f1A = 4;
    tPoints[89].link[0] = 0x5D;
    tPoints[89].link[1] = 0x5E;
    tPoints[89].link[2] = 0x76;
    tPoints[89].link[3] = 0x79;
    tPoints[90].vx = 0x2AA8;
    tPoints[90].vy = 0x12C0;
    tPoints[90].f1A = 3;
    tPoints[90].link[0] = 0x27;
    tPoints[90].link[1] = 0x63;
    tPoints[90].link[2] = 0x5B;
    tPoints[91].vx = 0x2BC0;
    tPoints[91].vy = 0x5A0;
    tPoints[91].f1A = 2;
    tPoints[91].link[0] = 0x5A;
    tPoints[91].link[1] = 0x5C;
    tPoints[92].vx = 0x3840;
    tPoints[92].vy = 0x758;
    tPoints[92].f1A = 3;
    tPoints[92].link[0] = 0x34;
    tPoints[92].link[1] = 0x5B;
    tPoints[92].link[2] = 0x5D;
    tPoints[93].vx = 0x4B00;
    tPoints[93].vy = 0x780;
    tPoints[93].f1A = 2;
    tPoints[93].link[0] = 0x5C;
    tPoints[93].link[1] = 0x5E;
    tPoints[94].vx = 0x5780;
    tPoints[94].vy = 0x5A0;
    tPoints[94].f1A = 3;
    tPoints[94].link[0] = 0x59;
    tPoints[94].link[1] = 0x5D;
    tPoints[94].link[2] = 0x5F;
    tPoints[95].vx = 0x5870;
    tPoints[95].vy = 0x12C0;
    tPoints[95].f1A = 3;
    tPoints[95].link[0] = 0x5E;
    tPoints[95].link[1] = 0x60;
    tPoints[95].link[2] = 0x47;
    tPoints[96].vx = 0x5730;
    tPoints[96].vy = 0x1F90;
    tPoints[96].f1A = 2;
    tPoints[96].link[0] = 0x5F;
    tPoints[96].link[1] = 0x61;
    tPoints[97].vx = 0x4B00;
    tPoints[97].vy = 0x1E00;
    tPoints[97].f1A = 2;
    tPoints[97].link[0] = 0x60;
    tPoints[97].link[1] = 0x62;
    tPoints[98].vx = 0x3840;
    tPoints[98].vy = 0x2030;
    tPoints[98].f1A = 2;
    tPoints[98].link[0] = 0x61;
    tPoints[98].link[1] = 0x63;
    tPoints[99].vx = 0x2BC0;
    tPoints[99].vy = 0x1F90;
    tPoints[99].f1A = 2;
    tPoints[99].link[0] = 0x5A;
    tPoints[99].link[1] = 0x62;
    Init3rdPartOfLevel5TriggerPts();
}

void Init3rdPartOfLevel5TriggerPts(void)
{
    s16 i;
    s16 j;

    for (i = 100; i < 124; i++) {
        for (j = 0; j < 4; j++) {
            tPoints[i].dist[j] = 0x4F;
        }
    }
    tPoints[100].vx = 0x24C8;
    tPoints[100].vy = 0x3930;
    tPoints[100].f1A = 2;
    tPoints[100].link[0] = 0x56;
    tPoints[100].link[1] = 0x65;
    tPoints[101].vx = 0x2390;
    tPoints[101].vy = 0x3C00;
    tPoints[101].f1A = 2;
    tPoints[101].link[0] = 0x64;
    tPoints[101].link[1] = 0x66;
    tPoints[102].vx = 0x2120;
    tPoints[102].vy = 0x3E00;
    tPoints[102].f1A = 2;
    tPoints[102].link[0] = 0x65;
    tPoints[102].link[1] = 0x7E;
    tPoints[103].vx = 0x2630;
    tPoints[103].vy = 0x3930;
    tPoints[103].f1A = 2;
    tPoints[103].link[0] = 0x56;
    tPoints[103].link[1] = 0x68;
    tPoints[104].vx = 0x2770;
    tPoints[104].vy = 0x3C00;
    tPoints[104].f1A = 2;
    tPoints[104].link[0] = 0x67;
    tPoints[104].link[1] = 0x69;
    tPoints[105].vx = 0x29E0;
    tPoints[105].vy = 0x3E00;
    tPoints[105].f1A = 2;
    tPoints[105].link[0] = 0x68;
    tPoints[105].link[1] = 0x7F;
    tPoints[106].vx = 0x4A50;
    tPoints[106].vy = 0x3930;
    tPoints[106].f1A = 2;
    tPoints[106].link[0] = 0x58;
    tPoints[106].link[1] = 0x6B;
    tPoints[107].vx = 0x4910;
    tPoints[107].vy = 0x3C00;
    tPoints[107].f1A = 2;
    tPoints[107].link[0] = 0x6A;
    tPoints[107].link[1] = 0x6C;
    tPoints[108].vx = 0x46A0;
    tPoints[108].vy = 0x3E00;
    tPoints[108].f1A = 2;
    tPoints[108].link[0] = 0x6B;
    tPoints[108].link[1] = 0x80;
    tPoints[109].vx = 0x4BB0;
    tPoints[109].vy = 0x3930;
    tPoints[109].f1A = 2;
    tPoints[109].link[0] = 0x58;
    tPoints[109].link[1] = 0x6E;
    tPoints[110].vx = 0x4CF0;
    tPoints[110].vy = 0x3C00;
    tPoints[110].f1A = 2;
    tPoints[110].link[0] = 0x6D;
    tPoints[110].link[1] = 0x6F;
    tPoints[111].vx = 0x4F60;
    tPoints[111].vy = 0x3E00;
    tPoints[111].f1A = 2;
    tPoints[111].link[0] = 0x6E;
    tPoints[111].link[1] = 0x81;
    tPoints[112].vx = 0x24D0;
    tPoints[112].vy = -0xF0;
    tPoints[112].f1A = 2;
    tPoints[112].link[0] = 0x57;
    tPoints[112].link[1] = 0x71;
    tPoints[113].vx = 0x2390;
    tPoints[113].vy = -0x3C0;
    tPoints[113].f1A = 2;
    tPoints[113].link[0] = 0x70;
    tPoints[113].link[1] = 0x72;
    tPoints[114].vx = 0x2120;
    tPoints[114].vy = -0x5C0;
    tPoints[114].f1A = 2;
    tPoints[114].link[0] = 0x71;
    tPoints[114].link[1] = 0x9A;
    tPoints[115].vx = 0x2630;
    tPoints[115].vy = -0xF0;
    tPoints[115].f1A = 2;
    tPoints[115].link[0] = 0x57;
    tPoints[115].link[1] = 0x74;
    tPoints[116].vx = 0x2770;
    tPoints[116].vy = -0x3C0;
    tPoints[116].f1A = 2;
    tPoints[116].link[0] = 0x73;
    tPoints[116].link[1] = 0x75;
    tPoints[117].vx = 0x29E0;
    tPoints[117].vy = -0x5C0;
    tPoints[117].f1A = 2;
    tPoints[117].link[0] = 0x74;
    tPoints[117].link[1] = 0x99;
    tPoints[118].vx = 0x4A50;
    tPoints[118].vy = -0xF0;
    tPoints[118].f1A = 2;
    tPoints[118].link[0] = 0x59;
    tPoints[118].link[1] = 0x77;
    tPoints[119].vx = 0x4910;
    tPoints[119].vy = -0x3C0;
    tPoints[119].f1A = 2;
    tPoints[119].link[0] = 0x76;
    tPoints[119].link[1] = 0x78;
    tPoints[120].vx = 0x46A0;
    tPoints[120].vy = -0x5C0;
    tPoints[120].f1A = 2;
    tPoints[120].link[0] = 0x77;
    tPoints[120].link[1] = 0x98;
    tPoints[121].vx = 0x4BB0;
    tPoints[121].vy = -0xF0;
    tPoints[121].f1A = 2;
    tPoints[121].link[0] = 0x59;
    tPoints[121].link[1] = 0x7A;
    tPoints[122].vx = 0x4CF0;
    tPoints[122].vy = -0x3C0;
    tPoints[122].f1A = 2;
    tPoints[122].link[0] = 0x79;
    tPoints[122].link[1] = 0x7B;
    tPoints[123].vx = 0x4F60;
    tPoints[123].vy = -0x5C0;
    tPoints[123].f1A = 2;
    tPoints[123].link[0] = 0x7A;
    tPoints[123].link[1] = 0x97;
    Init4thPartOfLevel5TriggerPts();
}

void Init4thPartOfLevel5TriggerPts(void)
{
    s16 i;
    s16 j;

    for (i = 124; i < 172; i++) {
        for (j = 0; j < 4; j++) {
            tPoints[i].dist[j] = 0x78;
        }
    }
    tPoints[124].vx = -0x600;
    tPoints[124].vy = 0x3FC0;
    tPoints[124].f1A = 2;
    tPoints[124].link[0] = 0x7D;
    tPoints[124].link[1] = 0xAC;
    tPoints[125].vx = 0xC80;
    tPoints[125].vy = 0x3FC0;
    tPoints[125].f1A = 2;
    tPoints[125].link[0] = 0x7C;
    tPoints[125].link[1] = 0x7E;
    tPoints[125].dist[0] = 0xA0;
    tPoints[125].dist[1] = 0xA0;
    tPoints[126].vx = 0x1E00;
    tPoints[126].vy = 0x3FC0;
    tPoints[126].f1A = 3;
    tPoints[126].link[0] = 0x66;
    tPoints[126].link[1] = 0x7D;
    tPoints[126].link[2] = 0x7F;
    tPoints[126].dist[1] = 0xA0;
    tPoints[126].dist[2] = 0xA0;
    tPoints[127].vx = 0x2D00;
    tPoints[127].vy = 0x3FC0;
    tPoints[127].f1A = 3;
    tPoints[127].link[0] = 0xB;
    tPoints[127].link[1] = 0x69;
    tPoints[127].link[2] = 0x7E;
    tPoints[127].dist[0] = 0xA0;
    tPoints[127].dist[2] = 0xA0;
    tPoints[128].vx = 0x4380;
    tPoints[128].vy = 0x3FC0;
    tPoints[128].f1A = 3;
    tPoints[128].link[0] = 0xB;
    tPoints[128].link[1] = 0x6C;
    tPoints[128].link[2] = 0x81;
    tPoints[128].dist[0] = 0xA0;
    tPoints[128].dist[2] = 0xA0;
    tPoints[129].vx = 0x5280;
    tPoints[129].vy = 0x3FC0;
    tPoints[129].f1A = 3;
    tPoints[129].link[0] = 0x6F;
    tPoints[129].link[1] = 0x80;
    tPoints[129].link[2] = 0x82;
    tPoints[129].dist[1] = 0xA0;
    tPoints[129].dist[2] = 0xA0;
    tPoints[130].vx = 0x60E0;
    tPoints[130].vy = 0x3FC0;
    tPoints[130].f1A = 2;
    tPoints[130].link[0] = 0x81;
    tPoints[130].link[1] = 0x83;
    tPoints[130].dist[0] = 0xA0;
    tPoints[130].dist[1] = 0xA0;
    tPoints[131].vx = 0x7680;
    tPoints[131].vy = 0x3FC0;
    tPoints[131].f1A = 2;
    tPoints[131].link[0] = 0x82;
    tPoints[131].link[1] = 0x84;
    tPoints[132].vx = 0x7880;
    tPoints[132].vy = 0x3EE0;
    tPoints[132].f1A = 2;
    tPoints[132].link[0] = 0x83;
    tPoints[132].link[1] = 0x85;
    tPoints[133].vx = 0x7950;
    tPoints[133].vy = 0x3CF0;
    tPoints[133].f1A = 2;
    tPoints[133].link[0] = 0x84;
    tPoints[133].link[1] = 0x86;
    tPoints[134].vx = 0x7950;
    tPoints[134].vy = 0x3200;
    tPoints[134].f1A = 2;
    tPoints[134].link[0] = 9;
    tPoints[134].link[1] = 0x85;
    tPoints[134].dist[0] = 0xA0;
    tPoints[134].dist[1] = 0xA0;
    tPoints[135].vx = 0x7950;
    tPoints[135].vy = 0x1D00;
    tPoints[135].f1A = 2;
    tPoints[135].link[0] = 9;
    tPoints[135].link[1] = 0x88;
    tPoints[136].vx = 0x7A20;
    tPoints[136].vy = 0x1B10;
    tPoints[136].f1A = 2;
    tPoints[136].link[0] = 0x87;
    tPoints[136].link[1] = 0x89;
    tPoints[136].dist[0] = 0x50;
    tPoints[136].dist[1] = 0x50;
    tPoints[137].vx = 0x7C20;
    tPoints[137].vy = 0x1A40;
    tPoints[137].f1A = 2;
    tPoints[137].link[0] = 0x88;
    tPoints[137].link[1] = 0x8A;
    tPoints[137].dist[0] = 0x50;
    tPoints[137].dist[1] = 0x50;
    tPoints[138].vx = 0x7E20;
    tPoints[138].vy = 0x1960;
    tPoints[138].f1A = 2;
    tPoints[138].link[0] = 0x89;
    tPoints[138].link[1] = 0x8B;
    tPoints[138].dist[0] = 0x50;
    tPoints[138].dist[1] = 0x50;
    tPoints[139].vx = 0x7EE8;
    tPoints[139].vy = 0x1770;
    tPoints[139].f1A = 2;
    tPoints[139].link[0] = 0x8A;
    tPoints[139].link[1] = 0x8C;
    tPoints[140].vx = 0x7EE8;
    tPoints[140].vy = 0x12F0;
    tPoints[140].f1A = 2;
    tPoints[140].link[0] = 0x8B;
    tPoints[140].link[1] = 0x8D;
    tPoints[141].vx = 0x7E10;
    tPoints[141].vy = 0x10F0;
    tPoints[141].f1A = 2;
    tPoints[141].link[0] = 0x8C;
    tPoints[141].link[1] = 0x8E;
    tPoints[141].dist[0] = 0x50;
    tPoints[141].dist[1] = 0x50;
    tPoints[142].vx = 0x7C10;
    tPoints[142].vy = 0x1018;
    tPoints[142].f1A = 2;
    tPoints[142].link[0] = 0x8D;
    tPoints[142].link[1] = 0x8F;
    tPoints[142].dist[0] = 0x50;
    tPoints[142].dist[1] = 0x50;
    tPoints[143].vx = 0x7A20;
    tPoints[143].vy = 0xF40;
    tPoints[143].f1A = 2;
    tPoints[143].link[0] = 0x8E;
    tPoints[143].link[1] = 0x90;
    tPoints[143].dist[0] = 0x50;
    tPoints[143].dist[1] = 0x50;
    tPoints[144].vx = 0x7950;
    tPoints[144].vy = 0xD50;
    tPoints[144].f1A = 2;
    tPoints[144].link[0] = 0x8F;
    tPoints[144].link[1] = 0x91;
    tPoints[145].vx = 0x7950;
    tPoints[145].vy = 0x578;
    tPoints[145].f1A = 2;
    tPoints[145].link[0] = 0x90;
    tPoints[145].link[1] = 0x92;
    tPoints[145].dist[0] = 0xA0;
    tPoints[145].dist[1] = 0xA0;
    tPoints[146].vx = 0x7950;
    tPoints[146].vy = -0x4B0;
    tPoints[146].f1A = 2;
    tPoints[146].link[0] = 0x91;
    tPoints[146].link[1] = 0x93;
    tPoints[147].vx = 0x7870;
    tPoints[147].vy = -0x6A0;
    tPoints[147].f1A = 2;
    tPoints[147].link[0] = 0x92;
    tPoints[147].link[1] = 0x94;
    tPoints[148].vx = 0x7680;
    tPoints[148].vy = -0x780;
    tPoints[148].f1A = 2;
    tPoints[148].link[0] = 0x93;
    tPoints[148].link[1] = 0x95;
    tPoints[149].vx = 0x6A40;
    tPoints[149].vy = -0x780;
    tPoints[149].f1A = 2;
    tPoints[149].link[0] = 0x94;
    tPoints[149].link[1] = 0x96;
    tPoints[149].dist[0] = 0xA0;
    tPoints[149].dist[1] = 0xA0;
    tPoints[150].vx = 0x5E88;
    tPoints[150].vy = -0x780;
    tPoints[150].f1A = 2;
    tPoints[150].link[0] = 0x95;
    tPoints[150].link[1] = 0x97;
    tPoints[150].dist[0] = 0xA0;
    tPoints[150].dist[1] = 0xA0;
    tPoints[151].vx = 0x5280;
    tPoints[151].vy = -0x780;
    tPoints[151].f1A = 3;
    tPoints[151].link[0] = 0x7B;
    tPoints[151].link[1] = 0x96;
    tPoints[151].link[2] = 0x98;
    tPoints[151].dist[1] = 0xA0;
    tPoints[151].dist[2] = 0xA0;
    tPoints[152].vx = 0x4380;
    tPoints[152].vy = -0x780;
    tPoints[152].f1A = 3;
    tPoints[152].link[0] = 0xC;
    tPoints[152].link[1] = 0x78;
    tPoints[152].link[2] = 0x97;
    tPoints[152].dist[1] = 0xA0;
    tPoints[152].dist[2] = 0xA0;
    tPoints[153].vx = 0x2D00;
    tPoints[153].vy = -0x780;
    tPoints[153].f1A = 3;
    tPoints[153].link[0] = 0xC;
    tPoints[153].link[1] = 0x75;
    tPoints[153].link[2] = 0x9A;
    tPoints[153].dist[0] = 0xA0;
    tPoints[153].dist[2] = 0xA0;
    tPoints[154].vx = 0x1E00;
    tPoints[154].vy = -0x780;
    tPoints[154].f1A = 3;
    tPoints[154].link[0] = 0x72;
    tPoints[154].link[1] = 0x99;
    tPoints[154].link[2] = 0x9B;
    tPoints[154].dist[1] = 0xA0;
    tPoints[154].dist[2] = 0xA0;
    tPoints[155].vx = 0xC80;
    tPoints[155].vy = -0x780;
    tPoints[155].f1A = 2;
    tPoints[155].link[0] = 0x9A;
    tPoints[155].link[1] = 0x9C;
    tPoints[155].dist[0] = 0xA0;
    tPoints[155].dist[1] = 0xA0;
    tPoints[156].vx = -0x600;
    tPoints[156].vy = -0x780;
    tPoints[156].f1A = 2;
    tPoints[156].link[0] = 0x9B;
    tPoints[156].link[1] = 0x9D;
    tPoints[157].vx = -0x800;
    tPoints[157].vy = -0x6A0;
    tPoints[157].f1A = 2;
    tPoints[157].link[0] = 0x9C;
    tPoints[157].link[1] = 0x9E;
    tPoints[158].vx = -0x8D0;
    tPoints[158].vy = -0x4B0;
    tPoints[158].f1A = 2;
    tPoints[158].link[0] = 0x9D;
    tPoints[158].link[1] = 0x9F;
    tPoints[159].vx = -0x8D0;
    tPoints[159].vy = 0x4B0;
    tPoints[159].f1A = 2;
    tPoints[159].link[0] = 0x9E;
    tPoints[159].link[1] = 0xA0;
    tPoints[159].dist[0] = 0xA0;
    tPoints[159].dist[1] = 0xA0;
    tPoints[160].vx = -0x8D0;
    tPoints[160].vy = 0xD50;
    tPoints[160].f1A = 2;
    tPoints[160].link[0] = 0x9F;
    tPoints[160].link[1] = 0xA1;
    tPoints[161].vx = -0x9A0;
    tPoints[161].vy = 0xF40;
    tPoints[161].f1A = 2;
    tPoints[161].link[0] = 0xA0;
    tPoints[161].link[1] = 0xA2;
    tPoints[161].dist[0] = 0x50;
    tPoints[161].dist[1] = 0x50;
    tPoints[162].vx = -0xBA0;
    tPoints[162].vy = 0x1020;
    tPoints[162].f1A = 2;
    tPoints[162].link[0] = 0xA1;
    tPoints[162].link[1] = 0xA3;
    tPoints[162].dist[0] = 0x50;
    tPoints[162].dist[1] = 0x50;
    tPoints[163].vx = -0xD90;
    tPoints[163].vy = 0x10F8;
    tPoints[163].f1A = 2;
    tPoints[163].link[0] = 0xA2;
    tPoints[163].link[1] = 0xA4;
    tPoints[163].dist[0] = 0x50;
    tPoints[163].dist[1] = 0x50;
    tPoints[164].vx = -0xE60;
    tPoints[164].vy = 0x12E0;
    tPoints[164].f1A = 2;
    tPoints[164].link[0] = 0xA3;
    tPoints[164].link[1] = 0xA5;
    tPoints[165].vx = -0xE60;
    tPoints[165].vy = 0x1760;
    tPoints[165].f1A = 2;
    tPoints[165].link[0] = 0xA4;
    tPoints[165].link[1] = 0xA6;
    tPoints[166].vx = -0xD90;
    tPoints[166].vy = 0x1970;
    tPoints[166].f1A = 2;
    tPoints[166].link[0] = 0xA5;
    tPoints[166].link[1] = 0xA7;
    tPoints[166].dist[0] = 0x50;
    tPoints[166].dist[1] = 0x50;
    tPoints[167].vx = -0xB80;
    tPoints[167].vy = 0x1A40;
    tPoints[167].f1A = 2;
    tPoints[167].link[0] = 0xA6;
    tPoints[167].link[1] = 0xA8;
    tPoints[167].dist[0] = 0x50;
    tPoints[167].dist[1] = 0x50;
    tPoints[168].vx = -0x9A0;
    tPoints[168].vy = 0x1B10;
    tPoints[168].f1A = 2;
    tPoints[168].link[0] = 0xA7;
    tPoints[168].link[1] = 0xA9;
    tPoints[168].dist[0] = 0x50;
    tPoints[168].dist[1] = 0x50;
    tPoints[169].vx = -0x8D0;
    tPoints[169].vy = 0x1D10;
    tPoints[169].f1A = 2;
    tPoints[169].link[0] = 0xA;
    tPoints[169].link[1] = 0xA8;
    tPoints[170].vx = -0x8D0;
    tPoints[170].vy = 0x3200;
    tPoints[170].f1A = 2;
    tPoints[170].link[0] = 0xA;
    tPoints[170].link[1] = 0xAB;
    tPoints[171].vx = -0x8D0;
    tPoints[171].vy = 0x3CE0;
    tPoints[171].f1A = 2;
    tPoints[171].link[0] = 0xAA;
    tPoints[171].link[1] = 0xAC;
    tPoints[172].vx = -0x7E8;
    tPoints[172].vy = 0x3EF0;
    tPoints[172].f1A = 2;
    tPoints[172].link[0] = 0x7C;
    tPoints[172].link[1] = 0xAB;
}

void InitLevel5Bridges(void)
{
    s16 i;
    s16 j;

    numBridges = 0x3A;
    numCheckBridges = 0x38;

    for (i = 0; i <= numBridges; i++) {
        bridges[i].unk08 = 0;
        bridges[i].unk0A = 10;
        for (j = 0; j < 4; j++) {
            bridges[i].obj[j] = -1;
        }
    }

    bridges[0].rampDir = 1;
    bridges[0].minX = 0x3780;
    bridges[0].minZ = 0x22E0;
    bridges[0].y0 = 0x90;
    bridges[0].y1 = 0;
    bridges[0].extentX = 0x180;
    bridges[0].extentZ = 0x180;
    bridges[0].obj[0] = 0x29;

    bridges[1].rampDir = 1;
    bridges[1].minX = 0x4A40;
    bridges[1].minZ = 0x22E0;
    bridges[1].y0 = 0x90;
    bridges[1].y1 = 0;
    bridges[1].extentX = 0x180;
    bridges[1].extentZ = 0x180;
    bridges[1].obj[0] = 0x2A;

    bridges[2].rampDir = 0;
    bridges[2].minX = 0x4A40;
    bridges[2].minZ = 0x120;
    bridges[2].y0 = 0;
    bridges[2].y1 = 0x90;
    bridges[2].extentX = 0x180;
    bridges[2].extentZ = 0x180;
    bridges[2].obj[0] = 0x2B;

    bridges[3].rampDir = 3;
    bridges[3].minX = 0x5B20;
    bridges[3].minZ = 0x1200;
    bridges[3].y0 = 0x90;
    bridges[3].y1 = 0;
    bridges[3].extentX = 0x180;
    bridges[3].extentZ = 0x180;
    bridges[3].obj[0] = 4;

    bridges[4].rampDir = 2;
    bridges[4].minX = 0x5A50;
    bridges[4].minZ = 0x1200;
    bridges[4].y0 = 0x78;
    bridges[4].y1 = 0x90;
    bridges[4].extentX = 0xD0;
    bridges[4].extentZ = 0x180;
    bridges[4].obj[0] = 3;
    bridges[4].obj[1] = 0x10;
    bridges[4].obj[2] = 0x11;
    bridges[4].obj[3] = 0x12;

    bridges[5].rampDir = 0;
    bridges[5].minX = 0x3780;
    bridges[5].minZ = 0x120;
    bridges[5].y0 = 0;
    bridges[5].y1 = 0x90;
    bridges[5].extentX = 0x180;
    bridges[5].extentZ = 0x180;
    bridges[5].obj[0] = 6;

    bridges[6].rampDir = 1;
    bridges[6].minX = 0x3780;
    bridges[6].minZ = 0x2A0;
    bridges[6].y0 = 0x90;
    bridges[6].y1 = 0x78;
    bridges[6].extentX = 0x180;
    bridges[6].extentZ = 0xD0;
    bridges[6].obj[0] = 5;
    bridges[6].obj[1] = 0x16;
    bridges[6].obj[2] = 0x17;
    bridges[6].obj[3] = 0x2E;

    bridges[7].rampDir = 2;
    bridges[7].minX = 0x26A0;
    bridges[7].minZ = 0x1200;
    bridges[7].y0 = 0;
    bridges[7].y1 = 0x90;
    bridges[7].extentX = 0x180;
    bridges[7].extentZ = 0x180;
    bridges[7].obj[0] = 8;

    bridges[8].rampDir = 3;
    bridges[8].minX = 0x2820;
    bridges[8].minZ = 0x1200;
    bridges[8].y0 = 0x90;
    bridges[8].y1 = 0x78;
    bridges[8].extentX = 0xD0;
    bridges[8].extentZ = 0x180;
    bridges[8].obj[0] = 7;
    bridges[8].obj[1] = 0xD;
    bridges[8].obj[2] = 0xE;
    bridges[8].obj[3] = 0xF;

    bridges[9].rampDir = 1;
    bridges[9].minX = 0x30C0;
    bridges[9].minZ = 0x1B28;
    bridges[9].y0 = 0x110;
    bridges[9].y1 = 0;
    bridges[9].extentX = 0x21C0;
    bridges[9].extentZ = 0x1D8;

    bridges[10].rampDir = 2;
    bridges[10].minX = 0x2E00;
    bridges[10].minZ = 0xB40;
    bridges[10].y0 = 0;
    bridges[10].y1 = 0x110;
    bridges[10].extentX = 0x1D8;
    bridges[10].extentZ = 0xF00;

    bridges[11].rampDir = 0;
    bridges[11].minX = 0x30C0;
    bridges[11].minZ = 0x880;
    bridges[11].y0 = 0;
    bridges[11].y1 = 0x110;
    bridges[11].extentX = 0x21C0;
    bridges[11].extentZ = 0x1D8;

    bridges[12].rampDir = 3;
    bridges[12].minX = 0x5368;
    bridges[12].minZ = 0xB40;
    bridges[12].y0 = 0x110;
    bridges[12].y1 = 0;
    bridges[12].extentX = 0x1D8;
    bridges[12].extentZ = 0xF00;

    bridges[13].rampDir = 3;
    bridges[13].minX = 0x27E8;
    bridges[13].minZ = 0x1380;
    bridges[13].y0 = 0x110;
    bridges[13].y1 = 0x78;
    bridges[13].extentX = 0x108;
    bridges[13].extentZ = 0xAE0;
    bridges[13].obj[0] = 8;
    bridges[13].obj[1] = 0xF;

    bridges[14].rampDir = 3;
    bridges[14].minX = 0x27E8;
    bridges[14].minZ = 0x720;
    bridges[14].y0 = 0x110;
    bridges[14].y1 = 0x78;
    bridges[14].extentX = 0x108;
    bridges[14].extentZ = 0xAE0;
    bridges[14].obj[0] = 8;
    bridges[14].obj[1] = 0xF;

    bridges[15].rampDir = 3;
    bridges[15].minX = 0x28F0;
    bridges[15].minZ = 0x720;
    bridges[15].y0 = 0x78;
    bridges[15].y1 = 0;
    bridges[15].extentX = 0xD0;
    bridges[15].extentZ = 0x1740;
    bridges[15].obj[0] = 8;
    bridges[15].obj[1] = 0xD;
    bridges[15].obj[2] = 0xE;

    bridges[16].rampDir = 3;
    bridges[16].minX = 0x5A50;
    bridges[16].minZ = 0x1380;
    bridges[16].y0 = 0x78;
    bridges[16].y1 = 0x110;
    bridges[16].extentX = 0x108;
    bridges[16].extentZ = 0xAE0;
    bridges[16].obj[0] = 4;
    bridges[16].obj[1] = 0x12;

    bridges[17].rampDir = 3;
    bridges[17].minX = 0x5A50;
    bridges[17].minZ = 0x720;
    bridges[17].y0 = 0x78;
    bridges[17].y1 = 0x110;
    bridges[17].extentX = 0x108;
    bridges[17].extentZ = 0xAE0;
    bridges[17].obj[0] = 4;
    bridges[17].obj[1] = 0x12;

    bridges[18].rampDir = 3;
    bridges[18].minX = 0x5980;
    bridges[18].minZ = 0x720;
    bridges[18].y0 = 0;
    bridges[18].y1 = 0x78;
    bridges[18].extentX = 0xD0;
    bridges[18].extentZ = 0x1740;
    bridges[18].obj[0] = 4;
    bridges[18].obj[1] = 0x10;
    bridges[18].obj[2] = 0x11;

    bridges[19].rampDir = 0;
    bridges[19].minX = 0x3900;
    bridges[19].minZ = 0x2210;
    bridges[19].y0 = 0x78;
    bridges[19].y1 = 0x110;
    bridges[19].extentX = 0x1140;
    bridges[19].extentZ = 0x108;
    bridges[19].obj[0] = 0x14;
    bridges[19].obj[1] = 0x15;
    bridges[19].obj[2] = 0x29;
    bridges[19].obj[3] = 0x2A;

    bridges[20].rampDir = 0;
    bridges[20].minX = 0x2CA0;
    bridges[20].minZ = 0x2140;
    bridges[20].y0 = 0;
    bridges[20].y1 = 0x78;
    bridges[20].extentX = 0x1440;
    bridges[20].extentZ = 0xD0;
    bridges[20].obj[0] = 0x13;
    bridges[20].obj[1] = 0x29;
    bridges[20].obj[2] = 0x2C;

    bridges[21].rampDir = 0;
    bridges[21].minX = 0x44A0;
    bridges[21].minZ = 0x2140;
    bridges[21].y0 = 0;
    bridges[21].y1 = 0x78;
    bridges[21].extentX = 0x1200;
    bridges[21].extentZ = 0xD0;
    bridges[21].obj[0] = 0x13;
    bridges[21].obj[1] = 0x2A;
    bridges[21].obj[2] = 0x2D;

    bridges[22].rampDir = 1;
    bridges[22].minX = 0x3900;
    bridges[22].minZ = 0x268;
    bridges[22].y0 = 0x110;
    bridges[22].y1 = 0x78;
    bridges[22].extentX = 0x1140;
    bridges[22].extentZ = 0x108;
    bridges[22].obj[0] = 6;
    bridges[22].obj[1] = 0x17;
    bridges[22].obj[2] = 0x18;
    bridges[22].obj[3] = 0x2B;

    bridges[23].rampDir = 1;
    bridges[23].minX = 0x2CA0;
    bridges[23].minZ = 0x370;
    bridges[23].y0 = 0x78;
    bridges[23].y1 = 0;
    bridges[23].extentX = 0x1200;
    bridges[23].extentZ = 0xD0;
    bridges[23].obj[0] = 6;
    bridges[23].obj[1] = 0x16;
    bridges[23].obj[2] = 0x2E;

    bridges[24].rampDir = 1;
    bridges[24].minX = 0x4260;
    bridges[24].minZ = 0x370;
    bridges[24].y0 = 0x78;
    bridges[24].y1 = 0;
    bridges[24].extentX = 0x1440;
    bridges[24].extentZ = 0xD0;
    bridges[24].obj[0] = 0x16;
    bridges[24].obj[1] = 0x2B;
    bridges[24].obj[2] = 0x2F;

    bridges[25].rampDir = 0;
    bridges[25].minX = 0x23C8;
    bridges[25].minZ = 0xC20;
    bridges[25].y0 = 8;
    bridges[25].y1 = 0x20;
    bridges[25].extentX = 0x90;
    bridges[25].extentZ = 0x48;
    bridges[25].obj[0] = 0x21;

    bridges[26].rampDir = 1;
    bridges[26].minX = 0x23C8;
    bridges[26].minZ = 0xCF8;
    bridges[26].y0 = 0x20;
    bridges[26].y1 = 8;
    bridges[26].extentX = 0x90;
    bridges[26].extentZ = 0x48;
    bridges[26].obj[0] = 0x21;

    bridges[27].rampDir = 2;
    bridges[27].minX = 0x250;
    bridges[27].minZ = 0x2690;
    bridges[27].y0 = 8;
    bridges[27].y1 = 0x20;
    bridges[27].extentX = 0x48;
    bridges[27].extentZ = 0x90;
    bridges[27].obj[0] = 0x22;

    bridges[28].rampDir = 3;
    bridges[28].minX = 0x328;
    bridges[28].minZ = 0x2690;
    bridges[28].y0 = 0x20;
    bridges[28].y1 = 8;
    bridges[28].extentX = 0x48;
    bridges[28].extentZ = 0x90;
    bridges[28].obj[0] = 0x22;

    bridges[29].rampDir = 2;
    bridges[29].minX = 0x6080;
    bridges[29].minZ = 0x120;
    bridges[29].y0 = 8;
    bridges[29].y1 = 0x20;
    bridges[29].extentX = 0x48;
    bridges[29].extentZ = 0x90;
    bridges[29].obj[0] = 0x23;

    bridges[30].rampDir = 3;
    bridges[30].minX = 0x6158;
    bridges[30].minZ = 0x120;
    bridges[30].y0 = 0x20;
    bridges[30].y1 = 8;
    bridges[30].extentX = 0x48;
    bridges[30].extentZ = 0x90;
    bridges[30].obj[0] = 0x23;

    bridges[30].rampDir = 2;
    bridges[30].minX = 0x5848;
    bridges[30].minZ = 0x34F8;
    bridges[30].y0 = 8;
    bridges[30].y1 = 0x20;
    bridges[30].extentX = 0x48;
    bridges[30].extentZ = 0x90;
    bridges[30].obj[0] = 0x24;

    bridges[32].rampDir = 3;
    bridges[32].minX = 0x5920;
    bridges[32].minZ = 0x34F8;
    bridges[32].y0 = 0x20;
    bridges[32].y1 = 8;
    bridges[32].extentX = 0x48;
    bridges[32].extentZ = 0x90;
    bridges[32].obj[0] = 0x24;

    bridges[33].rampDir = 0;
    bridges[33].minX = 0x23C8;
    bridges[33].minZ = 0xC68;
    bridges[33].y0 = 0x20;
    bridges[33].y1 = 0x20;
    bridges[33].extentX = 0x90;
    bridges[33].extentZ = 0x90;
    bridges[33].obj[0] = 0x1C;
    bridges[33].obj[1] = 0x1D;
    bridges[33].unk08 = 1;

    bridges[34].rampDir = 2;
    bridges[34].minX = 0x298;
    bridges[34].minZ = 0x2690;
    bridges[34].y0 = 0x20;
    bridges[34].y1 = 0x20;
    bridges[34].extentX = 0x90;
    bridges[34].extentZ = 0x90;
    bridges[34].obj[0] = 0x1E;
    bridges[34].obj[1] = 0x1F;
    bridges[34].unk08 = 2;

    bridges[35].rampDir = 2;
    bridges[35].minX = 0x60C8;
    bridges[35].minZ = 0x120;
    bridges[35].y0 = 0x20;
    bridges[35].y1 = 0x20;
    bridges[35].extentX = 0x90;
    bridges[35].extentZ = 0x90;
    bridges[35].obj[0] = 0x20;
    bridges[35].obj[1] = 0x21;
    bridges[35].unk08 = 3;

    bridges[36].rampDir = 2;
    bridges[36].minX = 0x5890;
    bridges[36].minZ = 0x34F8;
    bridges[36].y0 = 0x20;
    bridges[36].y1 = 0x20;
    bridges[36].extentX = 0x90;
    bridges[36].extentZ = 0x90;
    bridges[36].obj[0] = 0x22;
    bridges[36].obj[1] = 0x23;
    bridges[36].unk08 = 4;

    bridges[37].rampDir = 0;
    bridges[37].minX = -2656;
    bridges[37].minZ = 0x1E10;
    bridges[37].y0 = 0;
    bridges[37].y1 = 0xC0;
    bridges[37].extentX = 0x320;
    bridges[37].extentZ = 0x470;
    bridges[37].obj[0] = 0x38;

    bridges[38].rampDir = 1;
    bridges[38].minX = -2656;
    bridges[38].minZ = 0x2580;
    bridges[38].y0 = 0xC0;
    bridges[38].y1 = 0;
    bridges[38].extentX = 0x320;
    bridges[38].extentZ = 0x470;
    bridges[38].obj[0] = 0x38;

    bridges[39].rampDir = 0;
    bridges[39].minX = 0x77C0;
    bridges[39].minZ = 0x1E10;
    bridges[39].y0 = 0;
    bridges[39].y1 = 0xC0;
    bridges[39].extentX = 0x320;
    bridges[39].extentZ = 0x470;
    bridges[39].obj[0] = 0x39;

    bridges[40].rampDir = 1;
    bridges[40].minX = 0x77C0;
    bridges[40].minZ = 0x2580;
    bridges[40].y0 = 0xC0;
    bridges[40].y1 = 0;
    bridges[40].extentX = 0x320;
    bridges[40].extentZ = 0x470;
    bridges[40].obj[0] = 0x39;

    bridges[41].rampDir = 0;
    bridges[41].minX = 0x3780;
    bridges[41].minZ = 0x2210;
    bridges[41].y0 = 0x78;
    bridges[41].y1 = 0x90;
    bridges[41].extentX = 0x180;
    bridges[41].extentZ = 0xD0;
    bridges[41].obj[0] = 0;
    bridges[41].obj[1] = 0x13;
    bridges[41].obj[2] = 0x14;
    bridges[41].obj[3] = 0x2C;

    bridges[42].rampDir = 0;
    bridges[42].minX = 0x4A40;
    bridges[42].minZ = 0x2210;
    bridges[42].y0 = 0x78;
    bridges[42].y1 = 0x90;
    bridges[42].extentX = 0x180;
    bridges[42].extentZ = 0xD0;
    bridges[42].obj[0] = 1;
    bridges[42].obj[1] = 0x13;
    bridges[42].obj[2] = 0x15;
    bridges[42].obj[3] = 0x2D;

    bridges[43].rampDir = 1;
    bridges[43].minX = 0x4A40;
    bridges[43].minZ = 0x2A0;
    bridges[43].y0 = 0x90;
    bridges[43].y1 = 0x78;
    bridges[43].extentX = 0x180;
    bridges[43].extentZ = 0xD0;
    bridges[43].obj[0] = 2;
    bridges[43].obj[1] = 0x16;
    bridges[43].obj[2] = 0x18;
    bridges[43].obj[3] = 0x2F;

    bridges[44].rampDir = 0;
    bridges[44].minX = 0x2CA0;
    bridges[44].minZ = 0x2210;
    bridges[44].y0 = 0x78;
    bridges[44].y1 = 0x110;
    bridges[44].extentX = 0xAE0;
    bridges[44].extentZ = 0x108;
    bridges[44].obj[0] = 0x14;
    bridges[44].obj[1] = 0x29;

    bridges[45].rampDir = 0;
    bridges[45].minX = 0x4BC0;
    bridges[45].minZ = 0x2210;
    bridges[45].y0 = 0x78;
    bridges[45].y1 = 0x110;
    bridges[45].extentX = 0xAE0;
    bridges[45].extentZ = 0x108;
    bridges[45].obj[0] = 0x15;
    bridges[45].obj[1] = 0x2A;

    bridges[46].rampDir = 1;
    bridges[46].minX = 0x2CA0;
    bridges[46].minZ = 0x268;
    bridges[46].y0 = 0x110;
    bridges[46].y1 = 0x78;
    bridges[46].extentX = 0xAE0;
    bridges[46].extentZ = 0x108;
    bridges[46].obj[0] = 6;
    bridges[46].obj[1] = 0x17;

    bridges[47].rampDir = 1;
    bridges[47].minX = 0x4BC0;
    bridges[47].minZ = 0x268;
    bridges[47].y0 = 0x110;
    bridges[47].y1 = 0x78;
    bridges[47].extentX = 0xAE0;
    bridges[47].extentZ = 0x108;
    bridges[47].obj[0] = 0x18;
    bridges[47].obj[1] = 0x2B;

    bridges[48].rampDir = 0;
    bridges[48].minX = 0x2820;
    bridges[48].minZ = 0x1E60;
    bridges[48].y0 = 0xF0;
    bridges[48].y1 = 0xF0;
    bridges[48].extentX = 0x1A0;
    bridges[48].extentZ = 0x480;
    bridges[48].obj[0] = 0xD;
    bridges[48].obj[1] = 0xF;
    bridges[48].obj[2] = 0x31;

    bridges[49].rampDir = 3;
    bridges[49].minX = 0x2820;
    bridges[49].minZ = 0x2140;
    bridges[49].y0 = 0xF0;
    bridges[49].y1 = 0xF0;
    bridges[49].extentX = 0x480;
    bridges[49].extentZ = 0x1A0;
    bridges[49].obj[0] = 0x14;
    bridges[49].obj[1] = 0x2C;
    bridges[49].obj[2] = 0x30;

    bridges[50].rampDir = 0;
    bridges[50].minX = 0x5980;
    bridges[50].minZ = 0x1E60;
    bridges[50].y0 = 0xF0;
    bridges[50].y1 = 0xF0;
    bridges[50].extentX = 0x1A0;
    bridges[50].extentZ = 0x480;
    bridges[50].obj[0] = 0x10;
    bridges[50].obj[1] = 0x12;
    bridges[50].obj[2] = 0x33;

    bridges[51].rampDir = 3;
    bridges[51].minX = 0x56A0;
    bridges[51].minZ = 0x2140;
    bridges[51].y0 = 0xF0;
    bridges[51].y1 = 0xF0;
    bridges[51].extentX = 0x480;
    bridges[51].extentZ = 0x1A0;
    bridges[51].obj[0] = 0x15;
    bridges[51].obj[1] = 0x2D;
    bridges[51].obj[2] = 0x32;

    bridges[52].rampDir = 0;
    bridges[52].minX = 0x2820;
    bridges[52].minZ = 0x2A0;
    bridges[52].y0 = 0xF0;
    bridges[52].y1 = 0xF0;
    bridges[52].extentX = 0x1A0;
    bridges[52].extentZ = 0x480;
    bridges[52].obj[0] = 0xE;
    bridges[52].obj[1] = 0xF;
    bridges[52].obj[2] = 0x35;

    bridges[53].rampDir = 3;
    bridges[53].minX = 0x2820;
    bridges[53].minZ = 0x2A0;
    bridges[53].y0 = 0xF0;
    bridges[53].y1 = 0xF0;
    bridges[53].extentX = 0x480;
    bridges[53].extentZ = 0x1A0;
    bridges[53].obj[0] = 0x17;
    bridges[53].obj[1] = 0x2E;
    bridges[53].obj[2] = 0x34;

    bridges[54].rampDir = 0;
    bridges[54].minX = 0x5980;
    bridges[54].minZ = 0x2A0;
    bridges[54].y0 = 0xF0;
    bridges[54].y1 = 0xF0;
    bridges[54].extentX = 0x1A0;
    bridges[54].extentZ = 0x480;
    bridges[54].obj[0] = 0x11;
    bridges[54].obj[1] = 0x12;
    bridges[54].obj[2] = 0x37;

    bridges[55].rampDir = 3;
    bridges[55].minX = 0x56A0;
    bridges[55].minZ = 0x2A0;
    bridges[55].y0 = 0xF0;
    bridges[55].y1 = 0xF0;
    bridges[55].extentX = 0x480;
    bridges[55].extentZ = 0x1A0;
    bridges[55].obj[0] = 0x18;
    bridges[55].obj[1] = 0x2F;
    bridges[55].obj[2] = 0x36;

    bridges[56].rampDir = 0;
    bridges[56].minX = -2656;
    bridges[56].minZ = 0x2280;
    bridges[56].y0 = 0xC0;
    bridges[56].y1 = 0xC0;
    bridges[56].extentX = 0x320;
    bridges[56].extentZ = 0x300;
    bridges[56].obj[0] = 0x25;
    bridges[56].obj[1] = 0x26;

    bridges[57].rampDir = 0;
    bridges[57].minX = 0x77C0;
    bridges[57].minZ = 0x2280;
    bridges[57].y0 = 0xC0;
    bridges[57].y1 = 0xC0;
    bridges[57].extentX = 0x320;
    bridges[57].extentZ = 0x300;
    bridges[57].obj[0] = 0x27;
    bridges[57].obj[1] = 0x28;
}

void InitLevel5Curbs(void)
{
    numCurbs = 25;
    curbs[0].x = 192;
    curbs[0].z = 192;
    curbs[0].w = 2112;
    curbs[0].h = 4416;
    curbs[0].t = 8;

    curbs[1].x = 2496;
    curbs[1].z = 192;
    curbs[1].w = 2112;
    curbs[1].h = 4416;
    curbs[1].t = 8;

    curbs[2].x = 192;
    curbs[2].z = 4992;
    curbs[2].w = 4416;
    curbs[2].h = 2112;
    curbs[2].t = 8;

    curbs[3].x = 192;
    curbs[3].z = 7296;
    curbs[3].w = 4416;
    curbs[3].h = 2112;
    curbs[3].t = 8;

    curbs[4].x = 192;
    curbs[4].z = 9792;
    curbs[4].w = 2112;
    curbs[4].h = 4416;
    curbs[4].t = 8;

    curbs[5].x = 2496;
    curbs[5].z = 9792;
    curbs[5].w = 2112;
    curbs[5].h = 4416;
    curbs[5].t = 8;

    curbs[6].x = 4992;
    curbs[6].z = 192;
    curbs[6].w = 4416;
    curbs[6].h = 2112;
    curbs[6].t = 8;

    curbs[7].x = 4992;
    curbs[7].z = 2496;
    curbs[7].w = 4416;
    curbs[7].h = 2112;
    curbs[7].t = 8;

    curbs[8].x = 4992;
    curbs[8].z = 4992;
    curbs[8].w = 2112;
    curbs[8].h = 4416;
    curbs[8].t = 8;

    curbs[9].x = 7296;
    curbs[9].z = 4992;
    curbs[9].w = 2112;
    curbs[9].h = 4416;
    curbs[9].t = 8;

    curbs[10].x = 4992;
    curbs[10].z = 9792;
    curbs[10].w = 4416;
    curbs[10].h = 2112;
    curbs[10].t = 8;

    curbs[11].x = 4992;
    curbs[11].z = 12096;
    curbs[11].w = 4416;
    curbs[11].h = 2112;
    curbs[11].t = 8;

    curbs[12].x = 9792;
    curbs[12].z = 9792;
    curbs[12].w = 2112;
    curbs[12].h = 4416;
    curbs[12].t = 8;

    curbs[13].x = 12096;
    curbs[13].z = 9792;
    curbs[13].w = 2112;
    curbs[13].h = 4416;
    curbs[13].t = 8;

    curbs[14].x = 14592;
    curbs[14].z = 9792;
    curbs[14].w = 4416;
    curbs[14].h = 2112;
    curbs[14].t = 8;

    curbs[15].x = 14592;
    curbs[15].z = 12096;
    curbs[15].w = 4416;
    curbs[15].h = 2112;
    curbs[15].t = 8;

    curbs[16].x = 19392;
    curbs[16].z = 9792;
    curbs[16].w = 2112;
    curbs[16].h = 4416;
    curbs[16].t = 8;

    curbs[17].x = 21696;
    curbs[17].z = 9792;
    curbs[17].w = 2112;
    curbs[17].h = 4416;
    curbs[17].t = 8;

    curbs[18].x = 24192;
    curbs[18].z = 9792;
    curbs[18].w = 4416;
    curbs[18].h = 2112;
    curbs[18].t = 8;

    curbs[19].x = 24192;
    curbs[19].z = 12096;
    curbs[19].w = 4416;
    curbs[19].h = 2112;
    curbs[19].t = 8;

    curbs[20].x = 24192;
    curbs[20].z = 4992;
    curbs[20].w = 2112;
    curbs[20].h = 4416;
    curbs[20].t = 8;

    curbs[21].x = 26496;
    curbs[21].z = 4992;
    curbs[21].w = 2112;
    curbs[21].h = 4416;
    curbs[21].t = 8;

    curbs[22].x = 24192;
    curbs[22].z = 192;
    curbs[22].w = 4416;
    curbs[22].h = 2112;
    curbs[22].t = 8;

    curbs[23].x = 24192;
    curbs[23].z = 2496;
    curbs[23].w = 4416;
    curbs[23].h = 2112;
    curbs[23].t = 8;
}

void InitLevel5PotHoles(void)
{
    s16 i;
    s16 t;
    s32 n;

    numPotHoles = 2;
    numCheckPotHoles = 2;
    i = 0;
    do {
        potHoles[i].active = 0;
        t = i + 1;
        do {
        } while (0);
        n = numPotHoles;
        i = t;
    } while (t < n);

    potHoles[0].x = -2656;
    potHoles[0].z = 0x23A0;
    potHoles[0].a = 0x320;
    potHoles[0].b = 0xC0;
    potHoles[0].c = -16;

    potHoles[1].x = 0x77C0;
    potHoles[1].z = 0x23A0;
    potHoles[1].a = 0x320;
    potHoles[1].b = 0xC0;
    potHoles[1].c = -16;
}

s16 GetGroupTestPointIsIn(s32 x, s32 z, s32 y)
{
    s16 group;
    s32 i;
    s16 n;

    if (shellGetCurrentLevel() == LEVEL5_CYBURBIA || shellGetCurrentLevel() == LEVEL6_ROOFTOPS) {
        n = triggerPtGroups.count;
        group = n;
        for (i = 0; i < n; i++) {
            if (x < triggerPtGroups.pos[i].vx) {
                continue;
            }
            if (triggerPtGroups.pos[i].vx + triggerPtGroups.w[i] < x) {
                continue;
            }
            if (z < triggerPtGroups.pos[i].vy) {
                continue;
            }
            if (triggerPtGroups.pos[i].vy + triggerPtGroups.d[i] < z) {
                continue;
            }
            if (triggerPtGroups.pos[i].vz < y + 80) {
                group = i;
                break;
            }
            group = i;
        }
    } else {
        group = 0;
    }
    return group;
}

void PutAICarBackOnRoof(AICar* car)
{
    s32 dx;
    s32 dz;
    s32 tp;

    if (GetPlayerInfo(car->unk0E)->unkC0 == 8) {
        tp = 7;
    } else {
        tp = 0x1F;
    }
    dx = car->motion.pos.vx - tPoints[tp].vx;
    dz = car->motion.pos.vy - tPoints[tp].vy;
    if (abs(dx) < 240 && abs(dz) < 240) {
        car->stats.triggerPt = tp;
        InitTriggerPoint(car);
        car->stats.unk38 = 1;
        car->unk16F = 0;
    } else {
        car->motion.pos.vx = SmoothValue(car->motion.pos.vx, tPoints[tp].vx, 0x62);
        car->motion.pos.vy = SmoothValue(car->motion.pos.vy, tPoints[tp].vy, 0x62);
        car->stats.unk38 = 0;
        car->motion.vel.vy = 0;
        car->motion.rotDelta.vz = 0;
    }
}

void InitLevel6TriggerPoints(void)
{
    s16 i;
    s16 j;

    numTriggerPoints = 49;
    for (i = 0; i < numTriggerPoints; i++) {
        tPoints[i].f1A = 1;
        tPoints[i].type = 0;
        tPoints[i].f08 = 0;
        for (j = 0; j < 4; j++) {
            tPoints[i].link[j] = 0;
            tPoints[i].dist[j] = 0x79;
        }
    }
    for (i = 0; i < 15; i++) {
        triggerPtStartPts.start[i] = i;
        triggerPtStartPts.grp[i] = 0;
    }
    triggerPtStartPts.n = 6;
    triggerPtStartPts.start[0] = 0;
    triggerPtStartPts.start[1] = 0xA;
    triggerPtStartPts.start[2] = 0xF;
    triggerPtStartPts.start[3] = 0x12;
    triggerPtStartPts.start[4] = 0x1D;
    triggerPtStartPts.start[5] = 0x28;
    triggerPtStartPts.grp[0] = 0;
    triggerPtStartPts.grp[1] = 0;
    triggerPtStartPts.grp[2] = 0;
    triggerPtStartPts.grp[3] = 0;
    triggerPtStartPts.grp[4] = 0;
    triggerPtStartPts.grp[5] = 0;
    triggerPtGroups.count = 14;
    for (i = 0; i < triggerPtGroups.count; i++) {
        for (j = 0; j < triggerPtGroups.count; j++) {
            triggerPtGroups.a[i][j] = -1;
            triggerPtGroups.b[i][j] = 0;
            triggerPtGroups.minSpeed[i][j] = 0x3C;
        }
    }
    triggerPtGroups.pos[0].vx = 0xA0;
    triggerPtGroups.pos[0].vy = 0x50;
    triggerPtGroups.pos[0].vz = 0x1E50;
    triggerPtGroups.w[0] = 0x640;
    triggerPtGroups.d[0] = 0x12C0;
    triggerPtGroups.a[0][1] = 2;
    triggerPtGroups.a[0][3] = 2;
    triggerPtGroups.a[0][4] = 2;
    triggerPtGroups.a[0][5] = 2;
    triggerPtGroups.a[0][6] = 2;
    triggerPtGroups.a[0][7] = 2;
    triggerPtGroups.a[0][8] = 2;
    triggerPtGroups.a[0][9] = 2;
    triggerPtGroups.a[0][10] = 2;
    triggerPtGroups.a[0][11] = 2;
    triggerPtGroups.a[0][12] = 2;
    triggerPtGroups.b[0][1] = 2;
    triggerPtGroups.b[0][3] = 5;
    triggerPtGroups.b[0][4] = 5;
    triggerPtGroups.b[0][5] = 5;
    triggerPtGroups.b[0][6] = 5;
    triggerPtGroups.b[0][7] = 5;
    triggerPtGroups.b[0][8] = 5;
    triggerPtGroups.b[0][9] = 5;
    triggerPtGroups.b[0][10] = 2;
    triggerPtGroups.b[0][11] = 2;
    triggerPtGroups.b[0][12] = 2;
    triggerPtGroups.minSpeed[0][5] = 0x6E;
    triggerPtGroups.pos[1].vx = 0xD20;
    triggerPtGroups.pos[1].vy = 0x50;
    triggerPtGroups.pos[1].vz = 0x1E50;
    triggerPtGroups.w[1] = 0x640;
    triggerPtGroups.d[1] = 0x12C0;
    triggerPtGroups.a[1][0] = 4;
    triggerPtGroups.a[1][3] = 4;
    triggerPtGroups.a[1][4] = 4;
    triggerPtGroups.a[1][5] = 4;
    triggerPtGroups.a[1][6] = 4;
    triggerPtGroups.a[1][7] = 4;
    triggerPtGroups.a[1][8] = 4;
    triggerPtGroups.a[1][9] = 4;
    triggerPtGroups.a[1][10] = 4;
    triggerPtGroups.a[1][11] = 4;
    triggerPtGroups.a[1][12] = 4;
    triggerPtGroups.b[1][0] = 2;
    triggerPtGroups.b[1][3] = 5;
    triggerPtGroups.b[1][4] = 5;
    triggerPtGroups.b[1][5] = 5;
    triggerPtGroups.b[1][6] = 5;
    triggerPtGroups.b[1][7] = 2;
    triggerPtGroups.b[1][8] = 2;
    triggerPtGroups.b[1][9] = 2;
    triggerPtGroups.b[1][10] = 2;
    triggerPtGroups.b[1][11] = 2;
    triggerPtGroups.b[1][12] = 2;
    triggerPtGroups.minSpeed[1][5] = 0x6E;
    triggerPtGroups.pos[2].vx = 0x6E0;
    triggerPtGroups.pos[2].vy = 0x50;
    triggerPtGroups.pos[2].vz = 0x1E50;
    triggerPtGroups.w[2] = 0x640;
    triggerPtGroups.d[2] = 0x640;
    triggerPtGroups.a[2][3] = 0;
    triggerPtGroups.a[2][4] = 0;
    triggerPtGroups.a[2][5] = 0;
    triggerPtGroups.a[2][6] = 0;
    triggerPtGroups.a[2][7] = 0;
    triggerPtGroups.a[2][8] = 0;
    triggerPtGroups.a[2][9] = 0;
    triggerPtGroups.a[2][10] = 0;
    triggerPtGroups.a[2][11] = 0;
    triggerPtGroups.a[2][12] = 0;
    triggerPtGroups.b[2][3] = 1;
    triggerPtGroups.b[2][4] = 0;
    triggerPtGroups.b[2][5] = 1;
    triggerPtGroups.b[2][6] = 0;
    triggerPtGroups.b[2][7] = 0xA;
    triggerPtGroups.b[2][8] = 0xA;
    triggerPtGroups.b[2][9] = 0xA;
    triggerPtGroups.b[2][10] = 0xA;
    triggerPtGroups.b[2][11] = 0xA;
    triggerPtGroups.b[2][12] = 0xA;
    triggerPtGroups.minSpeed[2][10] = 0x78;
    triggerPtGroups.pos[3].vx = 0x6E0;
    triggerPtGroups.pos[3].vy = 0x2210;
    triggerPtGroups.pos[3].vz = 0x1D10;
    triggerPtGroups.w[3] = 0xC80;
    triggerPtGroups.d[3] = 0x640;
    triggerPtGroups.a[3][0] = 0xE;
    triggerPtGroups.a[3][1] = 0xE;
    triggerPtGroups.a[3][2] = 0xE;
    triggerPtGroups.a[3][6] = 0xE;
    triggerPtGroups.a[3][7] = 0xE;
    triggerPtGroups.a[3][8] = 0xE;
    triggerPtGroups.a[3][9] = 0xE;
    triggerPtGroups.a[3][10] = 0xE;
    triggerPtGroups.a[3][11] = 0xE;
    triggerPtGroups.a[3][12] = 0xE;
    triggerPtGroups.b[3][0] = 6;
    triggerPtGroups.b[3][1] = 6;
    triggerPtGroups.b[3][2] = 6;
    triggerPtGroups.b[3][6] = 6;
    triggerPtGroups.b[3][7] = 6;
    triggerPtGroups.b[3][8] = 6;
    triggerPtGroups.b[3][9] = 6;
    triggerPtGroups.b[3][10] = 6;
    triggerPtGroups.b[3][11] = 6;
    triggerPtGroups.b[3][12] = 6;
    triggerPtGroups.minSpeed[3][6] = 0x78;
    triggerPtGroups.pos[4].vx = 0x3C0;
    triggerPtGroups.pos[4].vy = 0x2080;
    triggerPtGroups.pos[4].vz = 0x1D10;
    triggerPtGroups.w[4] = 0x320;
    triggerPtGroups.d[4] = 0x7D0;
    triggerPtGroups.a[4][0] = 0xD;
    triggerPtGroups.a[4][1] = 0xD;
    triggerPtGroups.a[4][2] = 0xD;
    triggerPtGroups.a[4][6] = 0xD;
    triggerPtGroups.a[4][7] = 0xD;
    triggerPtGroups.a[4][8] = 0xD;
    triggerPtGroups.a[4][9] = 0xD;
    triggerPtGroups.a[4][10] = 0xD;
    triggerPtGroups.a[4][11] = 0xD;
    triggerPtGroups.a[4][12] = 0xD;
    triggerPtGroups.b[4][0] = 3;
    triggerPtGroups.b[4][1] = 3;
    triggerPtGroups.b[4][2] = 3;
    triggerPtGroups.b[4][6] = 3;
    triggerPtGroups.b[4][7] = 3;
    triggerPtGroups.b[4][8] = 3;
    triggerPtGroups.b[4][9] = 3;
    triggerPtGroups.b[4][10] = 3;
    triggerPtGroups.b[4][11] = 3;
    triggerPtGroups.b[4][12] = 3;
    triggerPtGroups.pos[5].vx = 0xA0;
    triggerPtGroups.pos[5].vy = 0x1590;
    triggerPtGroups.pos[5].vz = 0x1B30;
    triggerPtGroups.w[5] = 0x12C0;
    triggerPtGroups.d[5] = 0x12C0;
    triggerPtGroups.a[5][0] = 0xC;
    triggerPtGroups.a[5][1] = 0xC;
    triggerPtGroups.a[5][2] = 0xC;
    triggerPtGroups.a[5][3] = 0xC;
    triggerPtGroups.a[5][4] = 0xC;
    triggerPtGroups.a[5][6] = 0xC;
    triggerPtGroups.a[5][7] = 0xC;
    triggerPtGroups.a[5][8] = 0xC;
    triggerPtGroups.a[5][9] = 0xC;
    triggerPtGroups.a[5][10] = 0xC;
    triggerPtGroups.a[5][11] = 0xC;
    triggerPtGroups.a[5][12] = 0xC;
    triggerPtGroups.b[5][0] = 4;
    triggerPtGroups.b[5][1] = 4;
    triggerPtGroups.b[5][2] = 4;
    triggerPtGroups.b[5][3] = 4;
    triggerPtGroups.b[5][4] = 4;
    triggerPtGroups.b[5][6] = 4;
    triggerPtGroups.b[5][7] = 4;
    triggerPtGroups.b[5][8] = 4;
    triggerPtGroups.b[5][9] = 4;
    triggerPtGroups.b[5][10] = 4;
    triggerPtGroups.b[5][11] = 4;
    triggerPtGroups.b[5][12] = 4;
    triggerPtGroups.pos[6].vx = 0x15E0;
    triggerPtGroups.pos[6].vy = 0x1900;
    triggerPtGroups.pos[6].vz = 0x1C20;
    triggerPtGroups.w[6] = 0x960;
    triggerPtGroups.d[6] = 0xFA0;
    triggerPtGroups.a[6][0] = 0x11;
    triggerPtGroups.a[6][1] = 0x11;
    triggerPtGroups.a[6][2] = 0x11;
    triggerPtGroups.a[6][3] = 0x11;
    triggerPtGroups.a[6][4] = 0x11;
    triggerPtGroups.a[6][5] = 0x11;
    triggerPtGroups.a[6][7] = 0x11;
    triggerPtGroups.a[6][8] = 0x11;
    triggerPtGroups.a[6][9] = 0x11;
    triggerPtGroups.a[6][10] = 0x11;
    triggerPtGroups.a[6][11] = 0x11;
    triggerPtGroups.a[6][12] = 0x11;
    triggerPtGroups.b[6][0] = 8;
    triggerPtGroups.b[6][1] = 8;
    triggerPtGroups.b[6][2] = 8;
    triggerPtGroups.b[6][3] = 8;
    triggerPtGroups.b[6][4] = 8;
    triggerPtGroups.b[6][5] = 8;
    triggerPtGroups.b[6][7] = 8;
    triggerPtGroups.b[6][8] = 8;
    triggerPtGroups.b[6][9] = 8;
    triggerPtGroups.b[6][10] = 8;
    triggerPtGroups.b[6][11] = 8;
    triggerPtGroups.b[6][12] = 8;
    triggerPtGroups.minSpeed[6][5] = 0x78;
    triggerPtGroups.pos[7].vx = 0x2594;
    triggerPtGroups.pos[7].vy = 0xFB4;
    triggerPtGroups.pos[7].vz = 0x1CE8;
    triggerPtGroups.w[7] = 0x938;
    triggerPtGroups.d[7] = 0x938;
    triggerPtGroups.a[7][0] = 0x2B;
    triggerPtGroups.a[7][1] = 0x2B;
    triggerPtGroups.a[7][2] = 0x2B;
    triggerPtGroups.a[7][3] = 0x2B;
    triggerPtGroups.a[7][4] = 0x2B;
    triggerPtGroups.a[7][5] = 0x2B;
    triggerPtGroups.a[7][6] = 0x2B;
    triggerPtGroups.a[7][8] = 0x2B;
    triggerPtGroups.a[7][9] = 0x2B;
    triggerPtGroups.a[7][10] = 0x2B;
    triggerPtGroups.a[7][11] = 0x2B;
    triggerPtGroups.a[7][12] = 0x2B;
    triggerPtGroups.b[7][0] = 9;
    triggerPtGroups.b[7][1] = 9;
    triggerPtGroups.b[7][2] = 9;
    triggerPtGroups.b[7][3] = 9;
    triggerPtGroups.b[7][4] = 9;
    triggerPtGroups.b[7][5] = 9;
    triggerPtGroups.b[7][6] = 9;
    triggerPtGroups.b[7][8] = 9;
    triggerPtGroups.b[7][9] = 9;
    triggerPtGroups.b[7][10] = 9;
    triggerPtGroups.b[7][11] = 9;
    triggerPtGroups.b[7][12] = 9;
    triggerPtGroups.minSpeed[7][5] = 0x78;
    triggerPtGroups.pos[8].vx = 0x15E0;
    triggerPtGroups.pos[8].vy = 0x960;
    triggerPtGroups.pos[8].vz = 0x1F40;
    triggerPtGroups.w[8] = 0x28A0;
    triggerPtGroups.d[8] = 0x1F40;
    triggerPtGroups.a[8][0] = 0x19;
    triggerPtGroups.a[8][1] = 0x19;
    triggerPtGroups.a[8][2] = 0x19;
    triggerPtGroups.a[8][3] = 0x17;
    triggerPtGroups.a[8][4] = 0x17;
    triggerPtGroups.a[8][5] = 0x17;
    triggerPtGroups.a[8][6] = 0x16;
    triggerPtGroups.a[8][7] = 0x1B;
    triggerPtGroups.a[8][9] = 0x1B;
    triggerPtGroups.a[8][10] = 0x18;
    triggerPtGroups.a[8][11] = 0x1A;
    triggerPtGroups.a[8][12] = 0x1A;
    triggerPtGroups.b[8][0] = 1;
    triggerPtGroups.b[8][1] = 1;
    triggerPtGroups.b[8][2] = 1;
    triggerPtGroups.b[8][3] = 5;
    triggerPtGroups.b[8][4] = 5;
    triggerPtGroups.b[8][5] = 5;
    triggerPtGroups.b[8][6] = 6;
    triggerPtGroups.b[8][7] = 7;
    triggerPtGroups.b[8][9] = 7;
    triggerPtGroups.b[8][10] = 0xA;
    triggerPtGroups.b[8][11] = 0xB;
    triggerPtGroups.b[8][12] = 0xB;
    triggerPtGroups.minSpeed[8][1] = 0x78;
    triggerPtGroups.minSpeed[8][5] = 0x78;
    triggerPtGroups.pos[9].vx = 0x15E0;
    triggerPtGroups.pos[9].vy = 0x960;
    triggerPtGroups.pos[9].vz = 0x1CE8;
    triggerPtGroups.w[9] = 0x28A0;
    triggerPtGroups.d[9] = 0x1F40;
    triggerPtGroups.a[9][0] = 0x2E;
    triggerPtGroups.a[9][1] = 0x2E;
    triggerPtGroups.a[9][2] = 0x2E;
    triggerPtGroups.a[9][3] = 0x2E;
    triggerPtGroups.a[9][4] = 0x2E;
    triggerPtGroups.a[9][5] = 0x2E;
    triggerPtGroups.a[9][6] = 0x2E;
    triggerPtGroups.a[9][7] = 0x2B;
    triggerPtGroups.a[9][8] = 0x2E;
    triggerPtGroups.a[9][10] = 0x2E;
    triggerPtGroups.a[9][11] = 0x2E;
    triggerPtGroups.a[9][12] = 0x2E;
    triggerPtGroups.b[9][0] = 5;
    triggerPtGroups.b[9][1] = 5;
    triggerPtGroups.b[9][2] = 5;
    triggerPtGroups.b[9][3] = 5;
    triggerPtGroups.b[9][4] = 5;
    triggerPtGroups.b[9][5] = 5;
    triggerPtGroups.b[9][6] = 5;
    triggerPtGroups.b[9][7] = 7;
    triggerPtGroups.b[9][8] = 5;
    triggerPtGroups.b[9][10] = 5;
    triggerPtGroups.b[9][11] = 5;
    triggerPtGroups.b[9][12] = 5;
    triggerPtGroups.minSpeed[9][1] = 0x78;
    triggerPtGroups.minSpeed[9][2] = 0x78;
    triggerPtGroups.minSpeed[9][3] = 0x78;
    triggerPtGroups.minSpeed[9][4] = 0x78;
    triggerPtGroups.minSpeed[9][5] = 0x78;
    triggerPtGroups.minSpeed[9][6] = 0x78;
    triggerPtGroups.minSpeed[9][8] = 0x78;
    triggerPtGroups.minSpeed[9][10] = 0x78;
    triggerPtGroups.minSpeed[9][11] = 0x78;
    triggerPtGroups.minSpeed[9][12] = 0x78;
    triggerPtGroups.pos[10].vx = 0x15E0;
    triggerPtGroups.pos[10].vy = 0;
    triggerPtGroups.pos[10].vz = 0x1D60;
    triggerPtGroups.w[10] = 0x1900;
    triggerPtGroups.d[10] = 0x960;
    triggerPtGroups.a[10][0] = 0x25;
    triggerPtGroups.a[10][1] = 0x25;
    triggerPtGroups.a[10][2] = 0x25;
    triggerPtGroups.a[10][3] = 0x25;
    triggerPtGroups.a[10][4] = 0x25;
    triggerPtGroups.a[10][5] = 0x25;
    triggerPtGroups.a[10][6] = 0x25;
    triggerPtGroups.a[10][7] = 0x25;
    triggerPtGroups.a[10][8] = 0x25;
    triggerPtGroups.a[10][9] = 0x25;
    triggerPtGroups.a[10][11] = 0x25;
    triggerPtGroups.a[10][12] = 0x25;
    triggerPtGroups.b[10][0] = 0xB;
    triggerPtGroups.b[10][1] = 0xB;
    triggerPtGroups.b[10][2] = 0xB;
    triggerPtGroups.b[10][3] = 0xB;
    triggerPtGroups.b[10][4] = 0xB;
    triggerPtGroups.b[10][5] = 0xB;
    triggerPtGroups.b[10][6] = 0xB;
    triggerPtGroups.b[10][7] = 0xB;
    triggerPtGroups.b[10][8] = 0xB;
    triggerPtGroups.b[10][9] = 0xB;
    triggerPtGroups.b[10][11] = 0xB;
    triggerPtGroups.b[10][12] = 0xB;
    triggerPtGroups.pos[11].vx = 0x2EE0;
    triggerPtGroups.pos[11].vy = 0;
    triggerPtGroups.pos[11].vz = 0x1E50;
    triggerPtGroups.w[11] = 0xAF0;
    triggerPtGroups.d[11] = 0x960;
    triggerPtGroups.a[11][0] = 0x22;
    triggerPtGroups.a[11][1] = 0x22;
    triggerPtGroups.a[11][2] = 0x22;
    triggerPtGroups.a[11][3] = 0x22;
    triggerPtGroups.a[11][4] = 0x22;
    triggerPtGroups.a[11][5] = 0x22;
    triggerPtGroups.a[11][6] = 0x22;
    triggerPtGroups.a[11][7] = 0x22;
    triggerPtGroups.a[11][8] = 0x22;
    triggerPtGroups.a[11][9] = 0x22;
    triggerPtGroups.a[11][12] = 0x22;
    triggerPtGroups.b[11][0] = 8;
    triggerPtGroups.b[11][1] = 8;
    triggerPtGroups.b[11][2] = 8;
    triggerPtGroups.b[11][3] = 8;
    triggerPtGroups.b[11][4] = 8;
    triggerPtGroups.b[11][5] = 8;
    triggerPtGroups.b[11][6] = 8;
    triggerPtGroups.b[11][7] = 8;
    triggerPtGroups.b[11][8] = 8;
    triggerPtGroups.b[11][9] = 8;
    triggerPtGroups.b[11][12] = 0xC;
    triggerPtGroups.pos[12].vx = 0x39D1;
    triggerPtGroups.pos[12].vy = 0;
    triggerPtGroups.pos[12].vz = 0x1E50;
    triggerPtGroups.w[12] = 0x4B0;
    triggerPtGroups.d[12] = 0x960;
    triggerPtGroups.a[12][0] = 0x30;
    triggerPtGroups.a[12][1] = 0x30;
    triggerPtGroups.a[12][2] = 0x30;
    triggerPtGroups.a[12][3] = 0x30;
    triggerPtGroups.a[12][4] = 0x30;
    triggerPtGroups.a[12][5] = 0x30;
    triggerPtGroups.a[12][6] = 0x30;
    triggerPtGroups.a[12][7] = 0x30;
    triggerPtGroups.a[12][8] = 0x30;
    triggerPtGroups.a[12][9] = 0x30;
    triggerPtGroups.a[12][11] = 0x30;
    triggerPtGroups.b[12][0] = 8;
    triggerPtGroups.b[12][1] = 8;
    triggerPtGroups.b[12][2] = 8;
    triggerPtGroups.b[12][3] = 8;
    triggerPtGroups.b[12][4] = 8;
    triggerPtGroups.b[12][5] = 8;
    triggerPtGroups.b[12][6] = 8;
    triggerPtGroups.b[12][7] = 8;
    triggerPtGroups.b[12][8] = 8;
    triggerPtGroups.b[12][9] = 8;
    triggerPtGroups.b[12][11] = 0xB;
    triggerPtGroups.pos[13].vx = -0x1F40;
    triggerPtGroups.pos[13].vy = -0x1F40;
    triggerPtGroups.pos[13].vz = 0;
    triggerPtGroups.w[13] = 0x7D00;
    triggerPtGroups.d[13] = 0x6720;

    Init2ndPartOfLevel6TriggerPts();
}

void Init2ndPartOfLevel6TriggerPts(void)
{
    tPoints[0].vx = 0xA00;
    tPoints[0].vy = 0x370;
    tPoints[0].f1A = 4;
    tPoints[0].link[0] = 1;
    tPoints[0].link[1] = 2;
    tPoints[0].link[2] = 4;
    tPoints[0].link[3] = 0x25;
    tPoints[0].dist[3] = 0x4F;
    tPoints[0].type = 2;
    tPoints[1].vx = 0x320;
    tPoints[1].vy = 0x320;
    tPoints[1].f1A = 3;
    tPoints[1].link[0] = 0;
    tPoints[1].link[1] = 2;
    tPoints[1].link[2] = 3;
    tPoints[1].dist[0] = 0x4F;
    tPoints[1].dist[1] = 0x4F;
    tPoints[1].type = 0;
    tPoints[2].vx = 0x3C0;
    tPoints[2].vy = 0x640;
    tPoints[2].f1A = 4;
    tPoints[2].link[0] = 0;
    tPoints[2].link[1] = 1;
    tPoints[2].link[2] = 3;
    tPoints[2].link[3] = 0xA;
    tPoints[2].dist[0] = 0x4F;
    tPoints[2].dist[1] = 0x4F;
    tPoints[2].dist[2] = 0x4F;
    tPoints[2].dist[3] = 0x4F;
    tPoints[2].type = 0;
    tPoints[3].vx = 0x3C0;
    tPoints[3].vy = 0x1040;
    tPoints[3].f1A = 2;
    tPoints[3].link[0] = 1;
    tPoints[3].link[1] = 2;
    tPoints[3].type = 0;
    tPoints[4].vx = 0x1040;
    tPoints[4].vy = 0x640;
    tPoints[4].f1A = 4;
    tPoints[4].link[0] = 0;
    tPoints[4].link[1] = 5;
    tPoints[4].link[2] = 6;
    tPoints[4].link[3] = 0xC;
    tPoints[4].dist[0] = 0x4F;
    tPoints[4].dist[1] = 0x4F;
    tPoints[4].dist[2] = 0x4F;
    tPoints[4].dist[3] = 0x4F;
    tPoints[4].type = 1;
    tPoints[5].vx = 0x1040;
    tPoints[5].vy = 0x320;
    tPoints[5].f1A = 3;
    tPoints[5].link[0] = 0;
    tPoints[5].link[1] = 4;
    tPoints[5].link[2] = 6;
    tPoints[5].dist[0] = 0x4F;
    tPoints[5].dist[1] = 0x4F;
    tPoints[5].dist[2] = 0x4F;
    tPoints[5].type = 1;
    tPoints[6].vx = 0x1040;
    tPoints[6].vy = 0x1090;
    tPoints[6].f1A = 2;
    tPoints[6].link[0] = 4;
    tPoints[6].link[1] = 5;
    tPoints[6].type = 1;
    tPoints[7].vx = 0x230;
    tPoints[7].vy = 0x2580;
    tPoints[7].f1A = 1;
    tPoints[7].link[0] = 8;
    tPoints[7].type = 5;
    tPoints[8].vx = 0x230;
    tPoints[8].vy = 0x1DB0;
    tPoints[8].f1A = 3;
    tPoints[8].link[0] = 7;
    tPoints[8].link[1] = 9;
    tPoints[8].link[2] = 0xB;
    tPoints[8].type = 5;
    tPoints[9].vx = 0x4B0;
    tPoints[9].vy = 0x18B0;
    tPoints[9].f1A = 3;
    tPoints[9].link[0] = 8;
    tPoints[9].link[1] = 0xB;
    tPoints[9].link[2] = 0xC;
    tPoints[9].type = 5;
    tPoints[10].vx = 0x500;
    tPoints[10].vy = 0x1A90;
    tPoints[10].f1A = 2;
    tPoints[10].link[0] = 0xB;
    tPoints[10].link[1] = 0x2F;
    tPoints[10].type = 5;
    tPoints[11].vx = 0x1040;
    tPoints[11].vy = 0x17D0;
    tPoints[11].f1A = 4;
    tPoints[11].link[0] = 8;
    tPoints[11].link[1] = 0xA;
    tPoints[11].link[2] = 0xC;
    tPoints[11].link[3] = 0x2F;
    tPoints[11].dist[2] = 0x4F;
    tPoints[11].type = 5;
    tPoints[12].vx = 0x1220;
    tPoints[12].vy = 0x2120;
    tPoints[12].f1A = 4;
    tPoints[12].link[0] = 0xA;
    tPoints[12].link[1] = 0xB;
    tPoints[12].link[2] = 0xD;
    tPoints[12].link[3] = 0x2F;
    tPoints[12].dist[2] = 0x4F;
    tPoints[12].type = 5;
    tPoints[13].vx = 0x5A0;
    tPoints[13].vy = 0x2148;
    tPoints[13].f1A = 2;
    tPoints[13].link[0] = 0xC;
    tPoints[13].link[1] = 0xE;
    tPoints[13].dist[0] = 0x4F;
    tPoints[13].dist[1] = 0x4F;
    tPoints[13].dist[2] = 0x4F;
    tPoints[13].type = 4;
    tPoints[14].vx = 0x7D0;
    tPoints[14].vy = 0x2580;
    tPoints[14].f1A = 4;
    tPoints[14].link[0] = 0xD;
    tPoints[14].link[1] = 0xF;
    tPoints[14].link[2] = 0x10;
    tPoints[14].link[3] = 0x11;
    tPoints[14].dist[0] = 0x4F;
    tPoints[14].type = 3;
    tPoints[15].vx = 0xD20;
    tPoints[15].vy = 0x2580;
    tPoints[15].f1A = 2;
    tPoints[15].link[0] = 0xE;
    tPoints[15].link[1] = 0x10;
    tPoints[15].type = 3;
    tPoints[16].vx = 0x1090;
    tPoints[16].vy = 0x24E0;
    tPoints[16].f1A = 2;
    tPoints[16].link[0] = 0xE;
    tPoints[16].link[1] = 0xF;
    tPoints[16].type = 3;
    tPoints[17].vx = 0x1E50;
    tPoints[17].vy = 0x26C0;
    tPoints[17].f1A = 3;
    tPoints[17].link[0] = 0x12;
    tPoints[17].link[1] = 0x13;
    tPoints[17].link[2] = 0x16;
    tPoints[17].dist[0] = 0x4F;
    tPoints[17].dist[1] = 0x4F;
    tPoints[17].dist[2] = 0x4F;
    tPoints[17].type = 6;
    tPoints[18].vx = 0x1900;
    tPoints[18].vy = 0x1C20;
    tPoints[18].f1A = 2;
    tPoints[18].link[0] = 0x11;
    tPoints[18].link[1] = 0x14;
    tPoints[18].type = 6;
    tPoints[19].vx = 0x18B0;
    tPoints[19].vy = 0x1BD0;
    tPoints[19].f1A = 2;
    tPoints[19].link[0] = 0x11;
    tPoints[19].link[1] = 0x14;
    tPoints[19].type = 6;
    tPoints[20].vx = 0x18B0;
    tPoints[20].vy = 0x25D0;
    tPoints[20].f1A = 3;
    tPoints[20].link[0] = 0x11;
    tPoints[20].link[1] = 0x13;
    tPoints[20].link[2] = 0x15;
    tPoints[20].type = 6;
    tPoints[21].vx = 0x1B80;
    tPoints[21].vy = 0x1BD0;
    tPoints[21].f1A = 1;
    tPoints[21].link[0] = 0x14;
    tPoints[21].type = 6;
    tPoints[22].vx = 0x1E78;
    tPoints[22].vy = 0x1770;
    tPoints[22].f1A = 4;
    tPoints[22].link[0] = 0x11;
    tPoints[22].link[1] = 0x17;
    tPoints[22].link[2] = 0x19;
    tPoints[22].link[3] = 0x1D;
    tPoints[22].dist[0] = 0x4F;
    tPoints[22].dist[1] = 0x4F;
    tPoints[22].dist[2] = 0x4F;
    tPoints[22].dist[3] = 0x4F;
    tPoints[22].type = 8;
    tPoints[23].vx = 0x1900;
    tPoints[23].vy = 0x16D0;
    tPoints[23].f1A = 4;
    tPoints[23].link[0] = 0xA;
    tPoints[23].link[1] = 0x16;
    tPoints[23].link[2] = 0x18;
    tPoints[23].link[3] = 0x19;
    tPoints[23].dist[0] = 0x4F;
    tPoints[23].type = 8;
    tPoints[24].vx = 0x1860;
    tPoints[24].vy = 0xBB8;
    tPoints[24].f1A = 4;
    tPoints[24].link[0] = 0x16;
    tPoints[24].link[1] = 0x19;
    tPoints[24].link[2] = 0x1B;
    tPoints[24].link[3] = 0x26;
    tPoints[24].type = 8;
    tPoints[25].vx = 0x2260;
    tPoints[25].vy = 0x1090;
    tPoints[25].f1A = 4;
    tPoints[25].link[0] = 6;
    tPoints[25].link[1] = 0x16;
    tPoints[25].link[2] = 0x1B;
    tPoints[25].link[3] = 0x1E;
    tPoints[25].dist[0] = 0x4F;
    tPoints[25].type = 8;
    tPoints[26].vx = 0x3908;
    tPoints[26].vy = 0xAF0;
    tPoints[26].f1A = 4;
    tPoints[26].link[0] = 0x1B;
    tPoints[26].link[1] = 0x1C;
    tPoints[26].link[2] = 0x21;
    tPoints[26].link[3] = 0x22;
    tPoints[26].dist[3] = 0x4F;
    tPoints[26].type = 8;
    tPoints[27].vx = 0x28A0;
    tPoints[27].vy = 0xC80;
    tPoints[27].f1A = 4;
    tPoints[27].link[0] = 0x19;
    tPoints[27].link[1] = 0x1A;
    tPoints[27].link[2] = 0x1C;
    tPoints[27].link[3] = 0x29;
    tPoints[27].type = 8;
    tPoints[28].vx = 0x36B0;
    tPoints[28].vy = 0x12C0;
    tPoints[28].f1A = 4;
    tPoints[28].link[0] = 0x1A;
    tPoints[28].link[1] = 0x1B;
    tPoints[28].link[2] = 0x1D;
    tPoints[28].link[3] = 0x21;
    tPoints[28].type = 8;
    tPoints[29].vx = 0x2A30;
    tPoints[29].vy = 0x2260;
    tPoints[29].f1A = 4;
    tPoints[29].link[0] = 0x16;
    tPoints[29].link[1] = 0x1C;
    tPoints[29].link[2] = 0x1E;
    tPoints[29].link[3] = 0x21;
    tPoints[29].type = 8;
    tPoints[30].vx = 0x2260;
    tPoints[30].vy = 0x2580;
    tPoints[30].f1A = 3;
    tPoints[30].link[0] = 0x1D;
    tPoints[30].link[1] = 0x1F;
    tPoints[30].link[2] = 0x21;
    tPoints[30].type = 8;
    tPoints[31].vx = 0x3200;
    tPoints[31].vy = 0x2620;
    tPoints[31].f1A = 3;
    tPoints[31].link[0] = 0x1D;
    tPoints[31].link[1] = 0x1E;
    tPoints[31].link[2] = 0x20;
    tPoints[31].type = 8;
    tPoints[32].vx = 0x3B60;
    tPoints[32].vy = 0x2620;
    tPoints[32].f1A = 2;
    tPoints[32].link[0] = 0x1F;
    tPoints[32].link[1] = 0x21;
    tPoints[32].type = 8;
    tPoints[33].vx = 0x3B60;
    tPoints[33].vy = 0x1C20;
    tPoints[33].f1A = 3;
    tPoints[33].link[0] = 0x1C;
    tPoints[33].link[1] = 0x1D;
    tPoints[33].link[2] = 0x20;
    tPoints[33].type = 8;
    tPoints[34].vx = 0x3908;
    tPoints[34].vy = 0x280;
    tPoints[34].f1A = 3;
    tPoints[34].link[0] = 0x1A;
    tPoints[34].link[1] = 0x24;
    tPoints[34].link[2] = 0x30;
    tPoints[34].dist[0] = 0x4F;
    tPoints[34].type = 0xB;
    tPoints[35].vx = 0x3110;
    tPoints[35].vy = 0x258;
    tPoints[35].f1A = 3;
    tPoints[35].link[0] = 0x22;
    tPoints[35].link[1] = 0x24;
    tPoints[35].link[2] = 0x26;
    tPoints[35].type = 0xB;
    tPoints[36].vx = 0x3110;
    tPoints[36].vy = 0x898;
    tPoints[36].f1A = 3;
    tPoints[36].link[0] = 0x22;
    tPoints[36].link[1] = 0x23;
    tPoints[36].link[2] = 0x25;
    tPoints[36].dist[2] = 0x4F;
    tPoints[36].type = 0xB;
    tPoints[37].vx = 0x28A0;
    tPoints[37].vy = 0x870;
    tPoints[37].f1A = 4;
    tPoints[37].link[0] = 0x24;
    tPoints[37].link[1] = 0x26;
    tPoints[37].link[2] = 0x27;
    tPoints[37].link[3] = 0x28;
    tPoints[37].dist[0] = 0x4F;
    tPoints[37].type = 0xA;
    tPoints[38].vx = 0x1F40;
    tPoints[38].vy = 0x320;
    tPoints[38].f1A = 3;
    tPoints[38].link[0] = 0x25;
    tPoints[38].link[1] = 0x27;
    tPoints[38].link[2] = 0x28;
    tPoints[38].type = 0xA;
    tPoints[39].vx = 0x28A0;
    tPoints[39].vy = 0x320;
    tPoints[39].f1A = 3;
    tPoints[39].link[0] = 0x25;
    tPoints[39].link[1] = 0x26;
    tPoints[39].link[2] = 0x28;
    tPoints[39].type = 0xA;
    tPoints[40].vx = 0x1900;
    tPoints[40].vy = 0x690;
    tPoints[40].f1A = 3;
    tPoints[40].link[0] = 0x25;
    tPoints[40].link[1] = 0x26;
    tPoints[40].link[2] = 0x27;
    tPoints[40].type = 0xA;
    tPoints[41].vx = 0x2A30;
    tPoints[41].vy = 0x1450;
    tPoints[41].f1A = 1;
    tPoints[41].link[0] = 0x2A;
    tPoints[41].type = 7;
    tPoints[42].vx = 0x2BC0;
    tPoints[42].vy = 0x17D0;
    tPoints[42].f1A = 4;
    tPoints[42].link[0] = 0xB;
    tPoints[42].link[1] = 0x29;
    tPoints[42].link[2] = 0x2B;
    tPoints[42].link[3] = 0x2C;
    tPoints[42].type = 7;
    tPoints[43].vx = 0x2620;
    tPoints[43].vy = 0x17D0;
    tPoints[43].f1A = 3;
    tPoints[43].link[0] = 0xB;
    tPoints[43].link[1] = 0x2A;
    tPoints[43].link[2] = 0x2C;
    tPoints[43].dist[0] = 0x4F;
    tPoints[43].type = 7;
    tPoints[44].vx = 0x2260;
    tPoints[44].vy = 0x17D0;
    tPoints[44].f1A = 1;
    tPoints[44].link[0] = 0xB;
    tPoints[44].link[1] = 0x2B;
    tPoints[44].link[2] = 0x2D;
    tPoints[44].dist[0] = 0x4F;
    tPoints[44].type = 9;
    tPoints[45].vx = 0x1DB0;
    tPoints[45].vy = 0x17D0;
    tPoints[45].f1A = 1;
    tPoints[45].link[0] = 0xB;
    tPoints[45].link[1] = 0x2C;
    tPoints[45].link[2] = 0x2E;
    tPoints[45].dist[0] = 0x4F;
    tPoints[45].type = 9;
    tPoints[46].vx = 0x1900;
    tPoints[46].vy = 0x17D0;
    tPoints[46].f1A = 1;
    tPoints[46].link[0] = 0xB;
    tPoints[46].link[1] = 0x2D;
    tPoints[46].dist[0] = 0x4F;
    tPoints[46].type = 9;
    tPoints[47].vx = 0xFA0;
    tPoints[47].vy = 0x1C20;
    tPoints[47].f1A = 3;
    tPoints[47].link[0] = 0xA;
    tPoints[47].link[1] = 0xB;
    tPoints[47].link[2] = 0xC;
    tPoints[47].dist[2] = 0x4F;
    tPoints[47].type = 5;
    tPoints[48].vx = 0x3CF0;
    tPoints[48].vy = 0x7D0;
    tPoints[48].f1A = 1;
    tPoints[48].link[0] = 0x22;
    tPoints[48].dist[2] = 0x4F;
    tPoints[48].type = 0xC;
}

void InitLevel6Bridges(void)
{
    s16 i;
    s16 j;

    numBridges = 0xB;
    numCheckBridges = 0xA;

    for (i = 0; i <= numBridges; i++) {
        bridges[i].unk08 = 0;
        bridges[i].unk0A = 10;
        for (j = 0; j < 4; j++) {
            bridges[i].obj[j] = -1;
        }
    }

    bridges[0].rampDir = 3;
    bridges[0].minX = 0x6E1;
    bridges[0].minZ = 0x2080;
    bridges[0].y0 = 0x1E0;
    bridges[0].y1 = 0;
    bridges[0].extentX = 0x960;
    bridges[0].extentZ = 0x190;

    bridges[1].rampDir = 1;
    bridges[1].minX = 0x1DB0;
    bridges[1].minZ = 0x1900;
    bridges[1].y0 = 0x320;
    bridges[1].y1 = 0;
    bridges[1].extentX = 0x190;
    bridges[1].extentZ = 0xC80;

    bridges[2].rampDir = 2;
    bridges[2].minX = 0x2A30;
    bridges[2].minZ = 0x7D0;
    bridges[2].y0 = 0;
    bridges[2].y1 = 0xF0;
    bridges[2].extentX = 0x4B0;
    bridges[2].extentZ = 0x190;

    bridges[3].rampDir = 0;
    bridges[3].minX = 0x3840;
    bridges[3].minZ = 0x4AF;
    bridges[3].y0 = 0;
    bridges[3].y1 = 0xF0;
    bridges[3].extentX = 0x190;
    bridges[3].extentZ = 0x4B0;

    bridges[4].rampDir = 2;
    bridges[4].minX = 0x29A0;
    bridges[4].minZ = 0x1080;
    bridges[4].y0 = 0;
    bridges[4].y1 = 0x18;
    bridges[4].extentX = 0x48;
    bridges[4].extentZ = 0x90;
    bridges[4].obj[0] = 0xA;

    bridges[5].rampDir = 3;
    bridges[5].minX = 0x2A78;
    bridges[5].minZ = 0x1080;
    bridges[5].y0 = 0x18;
    bridges[5].y1 = 0;
    bridges[5].extentX = 0x48;
    bridges[5].extentZ = 0x90;
    bridges[5].obj[0] = 0xA;

    bridges[6].rampDir = 3;
    bridges[6].minX = 0x1638;
    bridges[6].minZ = 0xB18;
    bridges[6].y0 = 0x28;
    bridges[6].y1 = 0;
    bridges[6].extentX = 0xD0;
    bridges[6].extentZ = 0xA0;
    bridges[6].obj[0] = 7;

    bridges[7].rampDir = 2;
    bridges[7].minX = 0x1608;
    bridges[7].minZ = 0xB18;
    bridges[7].y0 = 0;
    bridges[7].y1 = 0x28;
    bridges[7].extentX = 0x30;
    bridges[7].extentZ = 0xA0;
    bridges[7].obj[0] = 6;

    bridges[8].rampDir = 3;
    bridges[8].minX = 0x1660;
    bridges[8].minZ = 0x1E0;
    bridges[8].y0 = 0x30;
    bridges[8].y1 = 0;
    bridges[8].extentX = 0xD0;
    bridges[8].extentZ = 0xA0;
    bridges[8].obj[0] = 9;

    bridges[9].rampDir = 2;
    bridges[9].minX = 0x1630;
    bridges[9].minZ = 0x1E0;
    bridges[9].y0 = 0;
    bridges[9].y1 = 0x30;
    bridges[9].extentX = 0x30;
    bridges[9].extentZ = 0xA0;
    bridges[9].obj[0] = 8;

    bridges[10].rampDir = 2;
    bridges[10].minX = 0x29E8;
    bridges[10].minZ = 0x1080;
    bridges[10].y0 = 0x18;
    bridges[10].y1 = 0x18;
    bridges[10].extentX = 0x90;
    bridges[10].extentZ = 0x90;
    bridges[10].obj[0] = 4;
    bridges[10].obj[1] = 5;
    bridges[10].unk08 = 1;
}
