#include "common.h"
#include "sdk/LIBAPI.H"
#include "tm1/TIMER.H"

extern s16 gFieldsLastFrame, gUpdateRate;
extern u32 gFrameCount, gTotalTics, gFrameTime, gLastTics, gCurTics;
static u32 ticks;

typedef struct TimerSnapshot {
    u32 frame;
    u32 elapsed_ticks;
    s32 fields;
    s32 update_rate;
} TimerSnapshot;

void tm1_timer_reset(void) {
    ticks = 0;
    gFieldsLastFrame = 2;
    gUpdateRate = 20;
    gTotalTics = gFrameTime = gLastTics = gCurTics = 0;
    InitTimer();
}

TimerSnapshot tm1_timer_step(u32 delta) {
    TimerSnapshot out;
    ticks += delta;
    SetFrameStart();
    out.frame = gFrameCount;
    out.elapsed_ticks = gFrameTime;
    out.fields = GetFieldsLastFrame();
    out.update_rate = GetUpdateRate();
    return out;
}

/* The harness injects total elapsed ticks; it does not model counter interrupts. */
u32 GetRCnt(u32 counter) { (void)counter; return ticks; }
void EnterCriticalSection(void) {}
void ExitCriticalSection(void) {}
u32 OpenEvent(u32 counter, u32 spec, u32 mode, void (*callback)(void)) {
    (void)counter; (void)spec; (void)mode; (void)callback;
    return 1;
}
void EnableEvent(u32 event) { (void)event; }
void DisableEvent(u32 event) { (void)event; }
void CloseEvent(u32 event) { (void)event; }
void SetRCnt(u32 counter, u32 value, u32 mode) {
    (void)counter; (void)value; (void)mode;
}
void StartRCnt(u32 counter) { (void)counter; }
void StopRCnt(u32 counter) { (void)counter; }
