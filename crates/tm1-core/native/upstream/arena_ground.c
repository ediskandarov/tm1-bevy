#include "tm1/arena_ground.h"
#include "tm1/arena_ai.h"

s16 numTriggerPoints, numBridges, numCheckBridges;
s16 numPotHoles, numCheckPotHoles, numCurbs, numSlickSpots;
TriggerPt tPoints[175];
TriggerPtGroups triggerPtGroups;
TriggerPtStartPts triggerPtStartPts;
Bridge bridges[59];
PotHole potHoles[5];
Curb curbs[8];
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
    tires[0].unk40 = triggerPtGroups.pos[group].vz;
    tires[1].unk40 = triggerPtGroups.pos[group].vz;
    tires[2].unk40 = triggerPtGroups.pos[group].vz;
    tires[3].unk40 = triggerPtGroups.pos[group].vz;
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
        value = triggerPtGroups.pos[g0].vz;
        previousHeight = triggerPtGroups.pos[previousGroup].vz;
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
                tires[0].unk24 = triggerPtGroups.pos[g0].vz;
            }
        } else if (value < 0) {
            tires[0].unk10 = g0;
            tires[0].unk0 = 1;
            tires[0].unk14 = 0;
            tires[0].unk26 = 0;
            tires[0].unk16 = tireOffset;
            tires[0].unk24 = triggerPtGroups.pos[g0].vz;
            tires[0].unk3 = 1;
        } else {
            tires[0].unk10 = g0;
        }
    }
    if (tires[1].unk10 != g1) {
        previousGroup = tires[1].unk10;
        value = triggerPtGroups.pos[g1].vz;
        previousHeight = triggerPtGroups.pos[previousGroup].vz;
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
                tires[1].unk24 = triggerPtGroups.pos[g1].vz;
            }
        } else if (value < 0) {
            tires[1].unk10 = g1;
            tires[1].unk0 = 1;
            tires[1].unk14 = 0;
            tires[1].unk26 = 0;
            tires[1].unk16 = tireOffset;
            tires[1].unk24 = triggerPtGroups.pos[g1].vz;
            tires[1].unk3 = 1;
        } else {
            tires[1].unk10 = g1;
        }
    }
    if (tires[2].unk10 != g2) {
        previousGroup = tires[2].unk10;
        value = triggerPtGroups.pos[g2].vz;
        previousHeight = triggerPtGroups.pos[previousGroup].vz;
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
                tires[2].unk24 = triggerPtGroups.pos[g2].vz;
            }
        } else if (value < 0) {
            tires[2].unk10 = g2;
            tires[2].unk0 = 1;
            tires[2].unk14 = 0;
            tires[2].unk26 = 0;
            tires[2].unk16 = tireOffset;
            tires[2].unk24 = triggerPtGroups.pos[g2].vz;
            tires[2].unk3 = 1;
        } else {
            tires[2].unk10 = g2;
        }
    }
    if (tires[3].unk10 != g3) {
        previousGroup = tires[3].unk10;
        value = triggerPtGroups.pos[g3].vz;
        previousHeight = triggerPtGroups.pos[previousGroup].vz;
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
                    tires[3].unk24 = triggerPtGroups.pos[g3].vz;
                }
            }
        } else if (value < 0) {
            tires[3].unk10 = g3;
            tires[3].unk0 = 1;
            tires[3].unk14 = 0;
            tires[3].unk26 = 0;
            tires[3].unk16 = tireOffset;
            tires[3].unk24 = triggerPtGroups.pos[g3].vz;
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
    s32 hi;
    s32 lo;
    s32 lim;
    s32 d0;
    s32 d1;
    s32 t;
    s32 p;
    s32 ang;
    s32 roll;
    s32 width;

    if (isPlayer) {
        m = &PLAYER_CAR(car)->motion;
        st = &PLAYER_CAR(car)->stats;
        tires = PLAYER_CAR(car)->tires;
        flags = PLAYER_CAR(car)->flags;
        bounce = &PLAYER_CAR(car)->bounce;
    } else {
        m = &AI_CAR(car)->motion;
        st = &AI_CAR(car)->stats;
        tires = AI_CAR(car)->tires;
        flags = AI_CAR(car)->flags;
        bounce = &AI_CAR(car)->bounce;
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
