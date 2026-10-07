/* MIT License. Copyright (c) 2026 Alex Spaulding */
#ifndef TIPA_BENCH_SUPPORT_H
#define TIPA_BENCH_SUPPORT_H

#include "../../include/iomfb.h"
#include <simd/simd.h>
#include <stdint.h>

#define TIPA_BENCH_FIELD_W 256
#define TIPA_BENCH_FIELD_H 256
#define TIPA_BENCH_PART_COUNT (TIPA_BENCH_FIELD_W * TIPA_BENCH_FIELD_H)
#define TIPA_BENCH_MAX_TOUCH 16

typedef struct {
    float time;
    float dt;
    vector_float2 res;
    vector_float2 cam;
    float load;
    uint32_t touch_n;
    vector_float4 touches[TIPA_BENCH_MAX_TOUCH];
    float fps;
    float gpu_ms;
    float cpu_ms;
    uint32_t thermal;
    float cpu_load;
    uint32_t frame;
    uint32_t steps;
} tipa_bench_uniforms;

typedef struct {
    vector_float2 p;
    vector_float2 v;
} tipa_bench_particle;

double tipa_bench_now_sec(void);
float tipa_bench_cpu_load_sample(void);

iomfb_present_fn tipa_bench_present_fn(void);

#endif
