// MIT License. Copyright (c) 2026 Alex Spaulding
// Heavy Metal demo for IOMFB zero-copy present. Original work.

#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float time;
    float dt;
    float2 res;
    float2 cam;
    float load;
    uint touch_n;
    float4 touches[16];
    float fps;
    float gpu_ms;
    float cpu_ms;
    uint thermal;
    float cpu_load;
    uint frame;
    uint steps;
};

struct Particle {
    float2 p;
    float2 v;
};

static float hash21(float2 p) {
    p = fract(p * float2(123.34, 345.45));
    p += dot(p, p + 34.345);
    return fract(p.x * p.y);
}

static float noise(float2 p) {
    float2 i = floor(p);
    float2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash21(i);
    float b = hash21(i + float2(1.0, 0.0));
    float c = hash21(i + float2(0.0, 1.0));
    float d = hash21(i + float2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

static float fbm(float2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 6; i++) {
        v += a * noise(p);
        p = p * 2.03 + 17.1;
        a *= 0.5;
    }
    return v;
}

static float sd_sphere(float3 p, float r) {
    return length(p) - r;
}

static float mandelbox(float3 p, int iters) {
    float3 z = p;
    float dr = 1.0;
    float scale = 2.4;
    float min_r = 0.5;
    float fixed_r = 1.0;
    for (int i = 0; i < 14; i++) {
        if (i >= iters) {
            break;
        }
        z = clamp(z, -1.0, 1.0) * 2.0 - z;
        float r2 = dot(z, z);
        if (r2 < min_r) {
            float t = fixed_r / min_r;
            z *= t;
            dr *= t;
        } else if (r2 < fixed_r) {
            float t = fixed_r / r2;
            z *= t;
            dr *= t;
        }
        z = z * scale + p;
        dr = dr * abs(scale) + 1.0;
    }
    return length(z) / abs(dr);
}

static float map_scene(float3 p, constant Uniforms &u) {
    int iters = 8 + int(u.load * 4.0);
    float d = mandelbox(p * 0.55, iters);
    d = min(d, p.y + 1.35);
    for (uint i = 0; i < 16u; i++) {
        if (i >= u.touch_n) {
            break;
        }
        float4 t = u.touches[i];
        if (t.w < 0.5) {
            continue;
        }
        float3 c = float3((t.x - 0.5) * 3.2, 0.35 + 0.25 * sin(u.time * 2.0 + t.z), (t.y - 0.5) * 3.2);
        float r = 0.18 + 0.08 * t.z;
        d = min(d, sd_sphere(p - c, r));
    }
    return d;
}

static float3 calc_normal(float3 p, constant Uniforms &u) {
    float2 e = float2(0.0014, 0.0);
    return normalize(float3(
        map_scene(p + e.xyy, u) - map_scene(p - e.xyy, u),
        map_scene(p + e.yxy, u) - map_scene(p - e.yxy, u),
        map_scene(p + e.yyx, u) - map_scene(p - e.yyx, u)));
}

static float soft_shadow(float3 ro, float3 rd, constant Uniforms &u) {
    float t = 0.04;
    float res = 1.0;
    int n = 12 + int(u.load * 10.0);
    for (int i = 0; i < 32; i++) {
        if (i >= n) {
            break;
        }
        float h = map_scene(ro + rd * t, u);
        res = min(res, 12.0 * h / t);
        t += clamp(h, 0.02, 0.25);
        if (res < 0.01 || t > 12.0) {
            break;
        }
    }
    return saturate(res);
}

static float ao(float3 p, float3 n, constant Uniforms &u) {
    float occ = 0.0;
    float sca = 1.0;
    for (int i = 0; i < 5; i++) {
        float h = 0.01 + 0.12 * float(i);
        occ += (h - map_scene(p + n * h, u)) * sca;
        sca *= 0.85;
    }
    return saturate(1.0 - 1.6 * occ);
}

static float3 camera_ray(float2 uv, float2 cam) {
    float yaw = cam.x;
    float pitch = cam.y;
    float3 fwd = normalize(float3(sin(yaw) * cos(pitch), sin(pitch), cos(yaw) * cos(pitch)));
    float3 right = normalize(cross(fwd, float3(0.0, 1.0, 0.0)));
    float3 up = cross(right, fwd);
    return normalize(fwd + uv.x * right * 0.9 + uv.y * up * 0.9);
}

static float3 sky(float3 rd, float time) {
    float h = saturate(rd.y * 0.5 + 0.5);
    float3 a = float3(0.02, 0.03, 0.08);
    float3 b = float3(0.55, 0.25, 0.85);
    float3 c = float3(0.95, 0.45, 0.15);
    float3 col = mix(a, b, h);
    col = mix(col, c, pow(saturate(rd.y * 0.35 + 0.15), 4.0));
    float sun = pow(saturate(dot(rd, normalize(float3(0.4, 0.55, 0.3)))), 48.0);
    col += float3(1.0, 0.7, 0.3) * sun;
    col += 0.08 * fbm(rd.xz * 4.0 + time * 0.05);
    return col;
}

fragment float4 fs_scene(float4 pos [[position]],
                         constant Uniforms &u [[buffer(0)]],
                         texture2d<float> field [[texture(0)]]) {
    constexpr sampler smp(address::repeat, filter::linear);
    float2 uv = (pos.xy - 0.5 * u.res) / min(u.res.x, u.res.y);
    uv.y = -uv.y;
    float3 ro = float3(sin(u.cam.x) * 3.4, 1.15 + 0.15 * sin(u.time * 0.4), cos(u.cam.x) * 3.4);
    float3 rd = camera_ray(uv, u.cam);

    float t = 0.0;
    float hit = -1.0;
    uint max_steps = max(u.steps, 48u);
    for (uint i = 0; i < 192u; i++) {
        if (i >= max_steps) {
            break;
        }
        float3 p = ro + rd * t;
        float d = map_scene(p, u);
        if (d < 0.0009) {
            hit = t;
            break;
        }
        t += d * 0.85;
        if (t > 28.0) {
            break;
        }
    }

    float3 col = sky(rd, u.time);
    if (hit > 0.0) {
        float3 p = ro + rd * hit;
        float3 n = calc_normal(p, u);
        float3 ldir = normalize(float3(0.55, 0.85, 0.25));
        float diff = saturate(dot(n, ldir));
        float sh = soft_shadow(p + n * 0.02, ldir, u);
        float occ = ao(p, n, u);
        float3 albedo = 0.22 + 0.22 * n + 0.15 * fbm(p.xz * 2.5 + u.time * 0.1);
        for (uint i = 0; i < 16u; i++) {
            if (i >= u.touch_n) {
                break;
            }
            float4 th = u.touches[i];
            if (th.w < 0.5) {
                continue;
            }
            float3 c = float3((th.x - 0.5) * 3.2, 0.4, (th.y - 0.5) * 3.2);
            float att = 1.6 / (1.0 + 8.0 * dot(p - c, p - c));
            albedo += float3(0.95, 0.25, 0.75) * att * th.z;
        }
        col = albedo * (0.12 + diff * sh) * occ;
        float3 h = normalize(ldir - rd);
        col += float3(0.8, 0.85, 1.0) * pow(saturate(dot(n, h)), 48.0) * sh;
        col = mix(sky(rd, u.time), col, exp(-0.035 * hit));
    }

    float2 fq = pos.xy / u.res;
    float4 fld = field.sample(smp, fq * 2.0 + u.time * 0.03);
    col += 0.07 * fld.rgb * (1.0 - saturate(hit * 0.2));
    col = pow(saturate(col), float3(0.85));
    return float4(col, 1.0);
}

static float digit(int n, float2 p) {
    p = (p - 0.5) * float2(1.6, 2.2);
    float2 a = abs(p);
    float d = 1.0;
    bool segs[7] = {false, false, false, false, false, false, false};
    switch (n) {
        case 0: segs[0]=segs[1]=segs[2]=segs[4]=segs[5]=segs[6]=true; break;
        case 1: segs[2]=segs[5]=true; break;
        case 2: segs[0]=segs[2]=segs[3]=segs[4]=segs[6]=true; break;
        case 3: segs[0]=segs[2]=segs[3]=segs[5]=segs[6]=true; break;
        case 4: segs[1]=segs[2]=segs[3]=segs[5]=true; break;
        case 5: segs[0]=segs[1]=segs[3]=segs[5]=segs[6]=true; break;
        case 6: segs[0]=segs[1]=segs[3]=segs[4]=segs[5]=segs[6]=true; break;
        case 7: segs[0]=segs[2]=segs[5]=true; break;
        case 8: segs[0]=segs[1]=segs[2]=segs[3]=segs[4]=segs[5]=segs[6]=true; break;
        case 9: segs[0]=segs[1]=segs[2]=segs[3]=segs[5]=segs[6]=true; break;
        default: break;
    }
    if (segs[0] && a.x < 0.35 && abs(p.y - 0.7) < 0.12) d = 0.0;
    if (segs[1] && a.y < 0.35 && abs(p.x + 0.38) < 0.12 && p.y > 0.0) d = 0.0;
    if (segs[2] && a.y < 0.35 && abs(p.x - 0.38) < 0.12 && p.y > 0.0) d = 0.0;
    if (segs[3] && a.x < 0.35 && abs(p.y) < 0.12) d = 0.0;
    if (segs[4] && a.y < 0.35 && abs(p.x + 0.38) < 0.12 && p.y < 0.0) d = 0.0;
    if (segs[5] && a.y < 0.35 && abs(p.x - 0.38) < 0.12 && p.y < 0.0) d = 0.0;
    if (segs[6] && a.x < 0.35 && abs(p.y + 0.7) < 0.12) d = 0.0;
    return d < 0.5 ? 1.0 : 0.0;
}

static float draw_int(int value, float2 uv, float2 origin, float2 cell) {
    float m = 0.0;
    int v = max(value, 0);
    for (int i = 3; i >= 0; i--) {
        int place = 1;
        for (int k = 0; k < i; k++) {
            place *= 10;
        }
        int dig = (v / place) % 10;
        float2 p = (uv - origin - float2(float(3 - i) * cell.x, 0.0)) / cell;
        if (p.x > 0.0 && p.x < 1.0 && p.y > 0.0 && p.y < 1.0) {
            m = max(m, digit(dig, p));
        }
    }
    return m;
}

fragment float4 fs_hud(float4 pos [[position]],
                       constant Uniforms &u [[buffer(0)]],
                       texture2d<float> scene [[texture(0)]]) {
    constexpr sampler smp(address::clamp_to_edge, filter::linear);
    float2 uv = pos.xy / u.res;
    float3 col = scene.sample(smp, uv).rgb;
    float2 hud = uv * float2(8.0, 18.0);
    float bar = 0.0;
    bar = max(bar, draw_int(int(u.fps + 0.5), hud, float2(0.15, 0.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.gpu_ms * 10.0 + 0.5), hud, float2(2.1, 0.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.cpu_ms * 10.0 + 0.5), hud, float2(4.05, 0.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.thermal), hud, float2(6.0, 0.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.cpu_load * 100.0 + 0.5), hud, float2(0.15, 1.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.load * 10.0 + 0.5), hud, float2(2.1, 1.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.touch_n), hud, float2(4.05, 1.25), float2(0.42, 0.85)));
    bar = max(bar, draw_int(int(u.steps), hud, float2(6.0, 1.25), float2(0.42, 0.85)));
    float3 tint = float3(0.2, 1.0, 0.55);
    if (u.thermal >= 2u) {
        tint = float3(1.0, 0.25, 0.2);
    } else if (u.thermal == 1u) {
        tint = float3(1.0, 0.8, 0.2);
    }
    col = mix(col, tint, bar * 0.92);
    for (uint i = 0; i < 16u; i++) {
        if (i >= u.touch_n) {
            break;
        }
        float4 t = u.touches[i];
        if (t.w < 0.5) {
            continue;
        }
        float d = length((uv - t.xy) * float2(u.res.x / u.res.y, 1.0));
        col += float3(1.0, 0.3, 0.8) * smoothstep(0.035, 0.0, d);
    }
    return float4(col, 1.0);
}

kernel void cs_field(texture2d<float, access::write> out [[texture(0)]],
                     device Particle *parts [[buffer(0)]],
                     constant Uniforms &u [[buffer(1)]],
                     uint2 gid [[thread_position_in_grid]]) {
    uint w = out.get_width();
    uint h = out.get_height();
    if (gid.x >= w || gid.y >= h) {
        return;
    }
    uint idx = gid.y * w + gid.x;
    float2 p = parts[idx].p;
    float2 v = parts[idx].v;
    float2 acc = 0.0;
    for (uint i = 0; i < 16u; i++) {
        if (i >= u.touch_n) {
            break;
        }
        float4 t = u.touches[i];
        if (t.w < 0.5) {
            continue;
        }
        float2 d = t.xy - p;
        float r2 = max(dot(d, d), 0.0004);
        acc += normalize(d) * (0.35 * t.z / r2);
    }
    float2 curl = float2(
        fbm(p * 6.0 + float2(0.0, u.time)) - 0.5,
        fbm(p * 6.0 + float2(4.2, u.time * 0.7)) - 0.5);
    acc += curl * (0.55 + 0.4 * u.load);
    v = (v + acc * u.dt) * 0.96;
    p = fract(p + v * u.dt);
    parts[idx].p = p;
    parts[idx].v = v;
    float glow = length(v) * 2.2 + fbm(p * 10.0 + u.time);
    float3 c = float3(0.15 + glow, 0.05 + 0.4 * p.y, 0.55 + 0.45 * p.x);
    for (uint i = 0; i < 16u; i++) {
        if (i >= u.touch_n) {
            break;
        }
        float4 t = u.touches[i];
        if (t.w < 0.5) {
            continue;
        }
        float d = distance(p, t.xy);
        c += float3(1.0, 0.2, 0.7) * exp(-18.0 * d);
    }
    out.write(float4(c, 1.0), gid);
}

vertex float4 vs_fullscreen(uint vid [[vertex_id]]) {
    float2 pos[3] = {float2(-1.0, -1.0), float2(3.0, -1.0), float2(-1.0, 3.0)};
    return float4(pos[vid], 0.0, 1.0);
}
