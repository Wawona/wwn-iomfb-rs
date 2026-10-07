/* MIT License. Copyright (c) 2026 Alex Spaulding */
#include "tipa_bench_support.h"
#include "../../include/iomfb.h"

#include <dispatch/dispatch.h>
#include <mach/mach_host.h>
#include <mach/mach_time.h>

double tipa_bench_now_sec(void) {
    static mach_timebase_info_data_t tb;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
        mach_timebase_info(&tb);
    });
    return (double)mach_absolute_time() * (double)tb.numer / (double)tb.denom / 1e9;
}

float tipa_bench_cpu_load_sample(void) {
    host_cpu_load_info_data_t info;
    mach_msg_type_number_t count = HOST_CPU_LOAD_INFO_COUNT;
    if (host_statistics(mach_host_self(), HOST_CPU_LOAD_INFO, (host_info_t)&info, &count)
        != KERN_SUCCESS) {
        return 0.0f;
    }
    static unsigned long long last_user, last_sys, last_idle, last_nice;
    unsigned long long user = info.cpu_ticks[CPU_STATE_USER];
    unsigned long long sys = info.cpu_ticks[CPU_STATE_SYSTEM];
    unsigned long long idle = info.cpu_ticks[CPU_STATE_IDLE];
    unsigned long long nice = info.cpu_ticks[CPU_STATE_NICE];
    unsigned long long du = user - last_user;
    unsigned long long ds = sys - last_sys;
    unsigned long long di = idle - last_idle;
    unsigned long long dn = nice - last_nice;
    last_user = user;
    last_sys = sys;
    last_idle = idle;
    last_nice = nice;
    unsigned long long tot = du + ds + di + dn;
    if (tot == 0) {
        return 0.0f;
    }
    return (float)(du + ds + dn) / (float)tot;
}

void tipa_bench_present_swift(
    void *ctx,
    void *surface,
    uint32_t w,
    uint32_t h,
    int32_t token,
    int32_t layer);

static void tipa_bench_present_trampoline(
    void *ctx,
    void *surface,
    uint32_t w,
    uint32_t h,
    int32_t token,
    int32_t layer) {
    tipa_bench_present_swift(ctx, surface, w, h, token, layer);
}

iomfb_present_fn tipa_bench_present_fn(void) {
    return tipa_bench_present_trampoline;
}
