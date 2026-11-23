#ifndef TM1_HOST_LIBAPI_H
#define TM1_HOST_LIBAPI_H
#include "common.h"
/* Keep the PSX SDK shims out of the host OS symbol namespace. */
#define EnterCriticalSection tm1_psx_enter_critical_section
#define ExitCriticalSection tm1_psx_exit_critical_section
#define OpenEvent tm1_psx_open_event
#define EnableEvent tm1_psx_enable_event
#define DisableEvent tm1_psx_disable_event
#define CloseEvent tm1_psx_close_event
#define SetRCnt tm1_psx_set_rcnt
#define StartRCnt tm1_psx_start_rcnt
#define StopRCnt tm1_psx_stop_rcnt
#define GetRCnt tm1_psx_get_rcnt

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
