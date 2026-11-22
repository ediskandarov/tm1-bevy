#ifndef TM1_HOST_TIMER_H
#define TM1_HOST_TIMER_H
#include "common.h"
void InitTimer(void);
void TermTimer(void);
u32 GetCurTics(void);
s16 GetFieldsLastFrame(void);
s32 GetUpdateRate(void);
u32 GetFrameCount(void);
void ResetUpdateRate(void);
void SetFrameStart(void);
#endif
