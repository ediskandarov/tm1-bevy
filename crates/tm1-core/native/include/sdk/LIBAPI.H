#ifndef TM1_HOST_LIBAPI_H
#define TM1_HOST_LIBAPI_H
#include "common.h"
void EnterCriticalSection(void);
void ExitCriticalSection(void);
u32 OpenEvent(u32 counter, u32 spec, u32 mode, void (*callback)(void));
void EnableEvent(u32 event);
void DisableEvent(u32 event);
void CloseEvent(u32 event);
void SetRCnt(u32 counter, u32 value, u32 mode);
void StartRCnt(u32 counter);
void StopRCnt(u32 counter);
u32 GetRCnt(u32 counter);
#endif
