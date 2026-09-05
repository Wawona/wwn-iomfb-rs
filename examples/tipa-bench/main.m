/* MIT License. Copyright (c) 2026 Alex Spaulding
 *
 * TrollStore Metal GPU bench. Encodes into an IOSurface-backed MTLTexture,
 * then iomfb-c presents that same IOSurface to the host view
 * (userspace, no IOMFB userclient).
 *
 * Touch: one finger orbits the camera. Extra fingers are lights / metaballs.
 * Pinch changes shader load. HUD: FPS, GPU ms, CPU ms, thermal, CPU %, load,
 * touch count, march steps.
 */
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#import <QuartzCore/QuartzCore.h>
#import <UIKit/UIKit.h>
#import <mach/mach_host.h>
#import <mach/mach_time.h>
#include <math.h>
#include <simd/simd.h>
#include <stdatomic.h>
#include <stdio.h>
#include <string.h>

#include "../../include/iomfb.h"

#define FIELD_W 256
#define FIELD_H 256
#define PART_COUNT (FIELD_W * FIELD_H)
#define MAX_TOUCH 16

typedef struct {
    float time;
    float dt;
    vector_float2 res;
    vector_float2 cam;
    float load;
    uint32_t touch_n;
    vector_float4 touches[MAX_TOUCH];
    float fps;
    float gpu_ms;
    float cpu_ms;
    uint32_t thermal;
    float cpu_load;
    uint32_t frame;
    uint32_t steps;
} Uniforms;

typedef struct {
    vector_float2 p;
    vector_float2 v;
} Particle;

static double now_sec(void) {
    static mach_timebase_info_data_t tb;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
        mach_timebase_info(&tb);
    });
    return (double)mach_absolute_time() * (double)tb.numer / (double)tb.denom / 1e9;
}

static float cpu_load_sample(void) {
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

static const char *thermal_name(NSProcessInfoThermalState s) {
    switch (s) {
        case NSProcessInfoThermalStateNominal:
            return "nominal";
        case NSProcessInfoThermalStateFair:
            return "fair";
        case NSProcessInfoThermalStateSerious:
            return "serious";
        case NSProcessInfoThermalStateCritical:
            return "critical";
        default:
            return "unknown";
    }
}

@interface BenchView : UIView
@end

static void bench_present(
    void *ctx, void *surface, uint32_t w, uint32_t h, int32_t token, int32_t layer) {
    (void)w;
    (void)h;
    (void)token;
    (void)layer;
    UIView *view = (__bridge UIView *)ctx;
    view.layer.contents = (__bridge id)surface;
}

@interface AppDelegate : UIResponder <UIApplicationDelegate>
@property (nonatomic, strong) UIWindow *window;
- (void)handleTouches:(NSSet<UITouch *> *)touches
                state:(int32_t)state
                event:(UIEvent *)event;
@end

@interface AppDelegate () {
    Uniforms _u;
}
@property (nonatomic, strong) CADisplayLink *link;
@property (nonatomic, strong) id<MTLDevice> device;
@property (nonatomic, strong) id<MTLCommandQueue> queue;
@property (nonatomic, strong) id<MTLRenderPipelineState> scenePipe;
@property (nonatomic, strong) id<MTLRenderPipelineState> hudPipe;
@property (nonatomic, strong) id<MTLComputePipelineState> fieldPipe;
@property (nonatomic, strong) id<MTLBuffer> uniformBuf;
@property (nonatomic, strong) id<MTLBuffer> particleBuf;
@property (nonatomic, strong) id<MTLTexture> fieldTex;
@property (nonatomic, strong) id<MTLTexture> sceneTex;
@property (nonatomic) void *swapchain;
@property (nonatomic) void *touch;
@property (nonatomic) uint32_t width;
@property (nonatomic) uint32_t height;
@property (nonatomic) double lastTime;
@property (nonatomic) double fpsEma;
@property (nonatomic) vector_float2 lastPan;
@property (nonatomic) BOOL hasPan;
@property (nonatomic) CGFloat pinch0;
@end

@implementation BenchView
- (void)touchesBegan:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    [(AppDelegate *)UIApplication.sharedApplication.delegate handleTouches:touches
                                                                    state:IOMFB_TOUCH_DOWN
                                                                    event:event];
}
- (void)touchesMoved:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    [(AppDelegate *)UIApplication.sharedApplication.delegate handleTouches:touches
                                                                    state:IOMFB_TOUCH_MOTION
                                                                    event:event];
}
- (void)touchesEnded:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    [(AppDelegate *)UIApplication.sharedApplication.delegate handleTouches:touches
                                                                    state:IOMFB_TOUCH_UP
                                                                    event:event];
}
- (void)touchesCancelled:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    [(AppDelegate *)UIApplication.sharedApplication.delegate handleTouches:touches
                                                                    state:IOMFB_TOUCH_CANCEL
                                                                    event:event];
}
@end

@implementation AppDelegate

- (void)handleTouches:(NSSet<UITouch *> *)touches
                state:(int32_t)state
                event:(UIEvent *)event {
    (void)event;
    if (!self.touch) {
        return;
    }
    CGSize vs = self.window.bounds.size;
    iomfb_touch_set_view(self.touch, vs.width, vs.height);
    uint32_t planted = 0;
    for (UITouch *t in touches) {
        CGPoint p = [t locationInView:self.window];
        iomfb_touch_event ev;
        memset(&ev, 0, sizeof(ev));
        int rc = iomfb_touch_inject(
            self.touch, (int32_t)t.hash, state, p.x, p.y, IOMFB_TOUCH_SPACE_VIEW, &ev);
        if (rc != IOMFB_C_OK) {
            continue;
        }
        if (ev.slot < MAX_TOUCH) {
            if (state == IOMFB_TOUCH_UP || state == IOMFB_TOUCH_CANCEL) {
                _u.touches[ev.slot] = simd_make_float4(ev.nx, ev.ny, 0.0f, 0.0f);
            } else {
                _u.touches[ev.slot] = simd_make_float4(ev.nx, ev.ny, 1.0f, 1.0f);
            }
        }
        if (state == IOMFB_TOUCH_DOWN) {
            self.lastPan = simd_make_float2((float)p.x, (float)p.y);
            self.hasPan = YES;
        } else if (state == IOMFB_TOUCH_MOTION && self.hasPan && touches.count == 1) {
            float dx = (float)p.x - self.lastPan.x;
            float dy = (float)p.y - self.lastPan.y;
            _u.cam.x += dx * 0.008f;
            _u.cam.y = fminf(1.1f, fmaxf(-0.35f, _u.cam.y - dy * 0.008f));
            self.lastPan = simd_make_float2((float)p.x, (float)p.y);
        }
        planted++;
    }
    if (touches.count >= 2) {
        NSArray<UITouch *> *all = touches.allObjects;
        if (all.count >= 2) {
            CGPoint a = [all[0] locationInView:self.window];
            CGPoint b = [all[1] locationInView:self.window];
            CGFloat dist = hypot(a.x - b.x, a.y - b.y);
            if (self.pinch0 < 1.0) {
                self.pinch0 = dist;
            } else {
                float scale = (float)(dist / self.pinch0);
                _u.load = fminf(2.2f, fmaxf(0.45f, _u.load * (0.7f + 0.3f * scale)));
                self.pinch0 = dist;
            }
        }
    } else {
        self.pinch0 = 0;
    }
    if (state == IOMFB_TOUCH_UP || state == IOMFB_TOUCH_CANCEL) {
        if (touches.count <= 1) {
            self.hasPan = NO;
        }
    }
    uint32_t n = 0;
    for (uint32_t i = 0; i < MAX_TOUCH; i++) {
        if (_u.touches[i].w > 0.5f) {
            n++;
        }
    }
    _u.touch_n = n;
    (void)planted;
}

- (BOOL)application:(UIApplication *)application
    didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    (void)application;
    (void)launchOptions;
    freopen("/tmp/iomfb-bench.log", "w", stderr);
    setvbuf(stderr, NULL, _IONBF, 0);

    self.window = [[UIWindow alloc] initWithFrame:UIScreen.mainScreen.bounds];
    BenchView *view = [[BenchView alloc] initWithFrame:self.window.bounds];
    view.multipleTouchEnabled = YES;
    view.backgroundColor = UIColor.blackColor;
    UIViewController *vc = [UIViewController new];
    vc.view = view;
    self.window.rootViewController = vc;
    [self.window makeKeyAndVisible];

    CGRect nb = UIScreen.mainScreen.nativeBounds;
    iomfb_display_configure((uint32_t)nb.size.width, (uint32_t)nb.size.height);

    void *sw = NULL;
    int rc = iomfb_swapchain_open(&sw);
    fprintf(stderr, "iomfb_swapchain_open rc=%d userland=%d bound=%u has_metal=%d\n",
        rc,
        sw ? iomfb_swapchain_is_userland(sw) : 0,
        iomfb_bound_export_count(),
        sw ? iomfb_swapchain_has_metal(sw) : 0);
    if (rc != IOMFB_C_OK || !sw) {
        return YES;
    }
    iomfb_swapchain_set_present(sw, bench_present, (__bridge void *)view);
    self.swapchain = sw;
    uint32_t w = 0, h = 0;
    iomfb_swapchain_size(sw, &w, &h);
    self.width = w;
    self.height = h;
    void *touch = NULL;
    iomfb_touch_open_swapchain(sw, &touch);
    self.touch = touch;
    iomfb_touch_set_view(touch, self.window.bounds.size.width, self.window.bounds.size.height);

    self.device = MTLCreateSystemDefaultDevice();
    fprintf(stderr, "MTLCreateSystemDefaultDevice=%s display=%ux%u\n",
        self.device ? [[self.device name] UTF8String] : "nil", w, h);
    if (!self.device) {
        return YES;
    }
    self.queue = [self.device newCommandQueue];
    NSString *libPath = [[NSBundle mainBundle] pathForResource:@"default" ofType:@"metallib"];
    NSError *err = nil;
    id<MTLLibrary> lib = [self.device newLibraryWithFile:libPath error:&err];
    if (!lib) {
        fprintf(stderr, "metallib load failed: %s\n", err.localizedDescription.UTF8String);
        return YES;
    }
    MTLRenderPipelineDescriptor *sceneD = [MTLRenderPipelineDescriptor new];
    sceneD.vertexFunction = [lib newFunctionWithName:@"vs_fullscreen"];
    sceneD.fragmentFunction = [lib newFunctionWithName:@"fs_scene"];
    sceneD.colorAttachments[0].pixelFormat = MTLPixelFormatBGRA8Unorm;
    self.scenePipe = [self.device newRenderPipelineStateWithDescriptor:sceneD error:&err];
    MTLRenderPipelineDescriptor *hudD = [MTLRenderPipelineDescriptor new];
    hudD.vertexFunction = [lib newFunctionWithName:@"vs_fullscreen"];
    hudD.fragmentFunction = [lib newFunctionWithName:@"fs_hud"];
    hudD.colorAttachments[0].pixelFormat = MTLPixelFormatBGRA8Unorm;
    self.hudPipe = [self.device newRenderPipelineStateWithDescriptor:hudD error:&err];
    id<MTLFunction> cs = [lib newFunctionWithName:@"cs_field"];
    self.fieldPipe = [self.device newComputePipelineStateWithFunction:cs error:&err];
    if (!self.scenePipe || !self.hudPipe || !self.fieldPipe) {
        fprintf(stderr, "pipeline failed: %s\n", err.localizedDescription.UTF8String);
        return YES;
    }

    MTLTextureDescriptor *fd = [MTLTextureDescriptor
        texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA16Float
                                     width:FIELD_W
                                    height:FIELD_H
                                 mipmapped:NO];
    fd.usage = MTLTextureUsageShaderRead | MTLTextureUsageShaderWrite;
    fd.storageMode = MTLStorageModeShared;
    self.fieldTex = [self.device newTextureWithDescriptor:fd];

    MTLTextureDescriptor *sd = [MTLTextureDescriptor
        texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
                                     width:w
                                    height:h
                                 mipmapped:NO];
    sd.usage = MTLTextureUsageShaderRead | MTLTextureUsageRenderTarget;
    sd.storageMode = MTLStorageModeShared;
    self.sceneTex = [self.device newTextureWithDescriptor:sd];

    self.uniformBuf = [self.device newBufferWithLength:sizeof(Uniforms)
                                               options:MTLResourceStorageModeShared];
    NSUInteger plen = sizeof(Particle) * PART_COUNT;
    self.particleBuf = [self.device newBufferWithLength:plen options:MTLResourceStorageModeShared];
    Particle *ps = (Particle *)self.particleBuf.contents;
    for (uint32_t i = 0; i < PART_COUNT; i++) {
        float x = (float)(i % FIELD_W) / (float)FIELD_W;
        float y = (float)(i / FIELD_W) / (float)FIELD_H;
        ps[i].p = simd_make_float2(x, y);
        ps[i].v = simd_make_float2(0, 0);
    }

    memset(&_u, 0, sizeof(_u));
    _u.res = simd_make_float2((float)w, (float)h);
    _u.cam = simd_make_float2(0.35f, 0.18f);
    _u.load = 1.0f;
    _u.steps = 96;
    _u.touch_n = 1;
    _u.touches[0] = simd_make_float4(0.5f, 0.42f, 1.0f, 1.0f);
    self.lastTime = now_sec();
    self.fpsEma = 60.0;

    self.link = [CADisplayLink displayLinkWithTarget:self selector:@selector(tick)];
    self.link.preferredFramesPerSecond = 60;
    [self.link addToRunLoop:NSRunLoop.mainRunLoop forMode:NSRunLoopCommonModes];
    fprintf(stderr, "bench start iosurface_zero_copy=1\n");
    dispatch_async(dispatch_get_main_queue(), ^{
        fprintf(stderr, "first tick dispatch\n");
        [self tick];
    });
    return YES;
}

- (void)tick {
    if (!self.swapchain || !self.queue) {
        return;
    }
    double t0 = now_sec();
    float dt = (float)fmax(1.0 / 240.0, t0 - self.lastTime);
    self.lastTime = t0;
    self.fpsEma = self.fpsEma * 0.9 + (1.0 / dt) * 0.1;
    _u.time += dt;
    _u.dt = dt;
    _u.fps = (float)self.fpsEma;
    _u.cpu_load = cpu_load_sample();
    NSProcessInfoThermalState th = NSProcessInfo.processInfo.thermalState;
    _u.thermal = (uint32_t)th;
    _u.steps = 64u + (uint32_t)(_u.load * 64.0f);
    if (_u.steps > 192u) {
        _u.steps = 192u;
    }
    _u.frame += 1;
    memcpy(self.uniformBuf.contents, &_u, sizeof(_u));

    void *surf = NULL;
    void *metal = NULL;
    uint32_t sid = 0;
    int arc = iomfb_swapchain_acquire(self.swapchain, &surf, &metal, &sid);
    if (arc != IOMFB_C_OK || !metal) {
        if ((_u.frame % 120u) == 0) {
            fprintf(stderr, "acquire rc=%d metal=%p\n", arc, metal);
        }
        return;
    }
    id<MTLTexture> presentTex = (__bridge id<MTLTexture>)metal;

    id<MTLCommandBuffer> buf = [self.queue commandBuffer];
    {
        id<MTLComputeCommandEncoder> enc = [buf computeCommandEncoder];
        [enc setComputePipelineState:self.fieldPipe];
        [enc setTexture:self.fieldTex atIndex:0];
        [enc setBuffer:self.particleBuf offset:0 atIndex:0];
        [enc setBuffer:self.uniformBuf offset:0 atIndex:1];
        MTLSize grid = MTLSizeMake(FIELD_W, FIELD_H, 1);
        NSUInteger tw = self.fieldPipe.threadExecutionWidth;
        NSUInteger th = self.fieldPipe.maxTotalThreadsPerThreadgroup / tw;
        if (th < 1) {
            th = 1;
        }
        [enc dispatchThreads:grid threadsPerThreadgroup:MTLSizeMake(tw, th, 1)];
        [enc endEncoding];
    }
    {
        MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
        pass.colorAttachments[0].texture = self.sceneTex;
        pass.colorAttachments[0].loadAction = MTLLoadActionClear;
        pass.colorAttachments[0].storeAction = MTLStoreActionStore;
        pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 1);
        id<MTLRenderCommandEncoder> enc = [buf renderCommandEncoderWithDescriptor:pass];
        [enc setRenderPipelineState:self.scenePipe];
        [enc setFragmentBuffer:self.uniformBuf offset:0 atIndex:0];
        [enc setFragmentTexture:self.fieldTex atIndex:0];
        [enc drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3];
        [enc endEncoding];
    }
    {
        MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
        pass.colorAttachments[0].texture = presentTex;
        pass.colorAttachments[0].loadAction = MTLLoadActionDontCare;
        pass.colorAttachments[0].storeAction = MTLStoreActionStore;
        id<MTLRenderCommandEncoder> enc = [buf renderCommandEncoderWithDescriptor:pass];
        [enc setRenderPipelineState:self.hudPipe];
        [enc setFragmentBuffer:self.uniformBuf offset:0 atIndex:0];
        [enc setFragmentTexture:self.sceneTex atIndex:0];
        [enc drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3];
        [enc endEncoding];
    }
    [buf commit];
    [buf waitUntilCompleted];
    double gpu = 0.0;
    if (buf.GPUEndTime > 0.0 && buf.GPUStartTime > 0.0) {
        gpu = (buf.GPUEndTime - buf.GPUStartTime) * 1000.0;
    }
    _u.gpu_ms = (float)gpu;
    iomfb_present_info info;
    memset(&info, 0, sizeof(info));
    int prc = iomfb_swapchain_present(self.swapchain, &info);
    double t1 = now_sec();
    _u.cpu_ms = (float)((t1 - t0) * 1000.0);
    if ((_u.frame % 15u) == 0) {
        fprintf(stderr,
            "frame=%u fps=%.1f gpu_ms=%.2f cpu_ms=%.2f thermal=%s(%u) cpu=%.0f%% "
            "load=%.2f steps=%u touches=%u sid=%u zero=%u setend=%d wait=%d token=%d\n",
            _u.frame, _u.fps, _u.gpu_ms, _u.cpu_ms, thermal_name(th),
            _u.thermal, _u.cpu_load * 100.0f, _u.load, _u.steps,
            _u.touch_n, sid, (unsigned)info.zero_copy, prc, info.wait_rc, info.token);
        fflush(stderr);
    }
}

@end

int main(int argc, char *argv[]) {
    @autoreleasepool {
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([AppDelegate class]));
    }
}
