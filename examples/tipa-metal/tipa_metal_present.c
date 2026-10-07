/* Metal GPU present proof. IOSurface == dma-buf == IOMFB SwapSetLayer.
 * UIKit process so the paravirt GPU can attach. No Apple IOMFB header.
 */
#import <CoreFoundation/CoreFoundation.h>
#import <Foundation/Foundation.h>
#import <IOKit/IOKitLib.h>
#import <IOSurface/IOSurfaceRef.h>
#import <Metal/Metal.h>
#import <UIKit/UIKit.h>
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

typedef void *Fb;
typedef int32_t (*FnGet)(Fb *);
typedef int32_t (*FnSize)(Fb, double *);
typedef int32_t (*FnBegin)(Fb, int *);
typedef int32_t (*FnEnd)(Fb);
typedef int32_t (*FnWait)(Fb, int, int);
typedef int32_t (*FnSetLayer)(
    Fb, int, IOSurfaceRef, double, double, double, double, double, double, double, double, int);
typedef int32_t (*FnDefault)(Fb, int, IOSurfaceRef *);
typedef int32_t (*FnPower)(Fb, int);
typedef int32_t (*FnPowerSave)(Fb, int);

static const char *kPublicExports[] = {
#include "../tipa-smoke/exports.inc"
};

static void *must_dlsym(void *h, const char *n) {
    void *p = dlsym(h, n);
    if (!p) {
        fprintf(stderr, "missing %s\n", n);
    }
    return p;
}

static void probe_gpu_services(void) {
    const char *names[] = {
        "IOGPUDevice",
        "IOGPU",
        "AGXAccelerator",
        "AppleParavirtGPU",
        "AppleParavirtGPUDevice",
        "IOMobileFramebuffer",
        NULL,
    };
    for (int i = 0; names[i]; i++) {
        io_iterator_t it = IO_OBJECT_NULL;
        CFDictionaryRef match = IOServiceMatching(names[i]);
        kern_return_t kr = IOServiceGetMatchingServices(kIOMainPortDefault, match, &it);
        int n = 0;
        if (kr == KERN_SUCCESS && it) {
            io_object_t obj;
            while ((obj = IOIteratorNext(it)) != IO_OBJECT_NULL) {
                n++;
                IOObjectRelease(obj);
            }
            IOObjectRelease(it);
        }
        fprintf(stderr, "iogpu_match %s kr=%d count=%d\n", names[i], (int)kr, n);
    }
}

static id<MTLDevice> pick_metal_device(void) {
    id<MTLDevice> sys = MTLCreateSystemDefaultDevice();
    fprintf(stderr, "MTLCreateSystemDefaultDevice=%s\n",
        sys ? [[sys name] UTF8String] : "nil");
    return sys;
}

int tipa_metal_present_frame(void) {
    void *lib = dlopen(
        "/System/Library/PrivateFrameworks/IOMobileFramebuffer.framework/IOMobileFramebuffer",
        RTLD_LAZY);
    if (!lib) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 1;
    }
    size_t n = sizeof(kPublicExports) / sizeof(kPublicExports[0]);
    size_t bound = 0;
    for (size_t i = 0; i < n; i++) {
        if (dlsym(lib, kPublicExports[i])) {
            bound++;
        }
    }
    fprintf(stderr, "bind %zu/%zu\n", bound, n);
    probe_gpu_services();

    FnGet get_main = must_dlsym(lib, "IOMobileFramebufferGetMainDisplay");
    FnGet get_sec = must_dlsym(lib, "IOMobileFramebufferGetSecondaryDisplay");
    FnSize get_size = must_dlsym(lib, "IOMobileFramebufferGetDisplaySize");
    FnBegin begin = must_dlsym(lib, "IOMobileFramebufferSwapBegin");
    FnEnd end = must_dlsym(lib, "IOMobileFramebufferSwapEnd");
    FnWait wait = must_dlsym(lib, "IOMobileFramebufferSwapWait");
    FnSetLayer set_layer = must_dlsym(lib, "IOMobileFramebufferSwapSetLayer");
    FnDefault get_def = must_dlsym(lib, "IOMobileFramebufferGetLayerDefaultSurface");
    FnPower power = must_dlsym(lib, "IOMobileFramebufferRequestPowerChange");
    FnPowerSave power_save = must_dlsym(lib, "IOMobileFramebufferEnableDisableVideoPowerSavings");
    if (!get_main || !get_size || !begin || !end || !wait || !set_layer) {
        return 2;
    }

    Fb fb = NULL;
    int32_t rc = get_main(&fb);
    if (rc != 0 || !fb) {
        if (get_sec) {
            rc = get_sec(&fb);
        }
    }
    if (rc != 0 || !fb) {
        fprintf(stderr, "GetMain/Secondary failed %d\n", rc);
        return 3;
    }
    double size[2] = {0, 0};
    rc = get_size(fb, size);
    if (rc != 0) {
        fprintf(stderr, "GetDisplaySize failed %d\n", rc);
        return 4;
    }
    int width = (int)size[0];
    int height = (int)size[1];
    fprintf(stderr, "display %dx%d\n", width, height);
    if (power) {
        fprintf(stderr, "RequestPowerChange(1)=%d\n", power(fb, 1));
    }
    if (power_save) {
        fprintf(stderr, "EnableDisableVideoPowerSavings(0)=%d\n", power_save(fb, 0));
    }

    int32_t pix = 'BGRA';
    int four = 4;
    CFStringRef keys[] = {
        kIOSurfaceWidth,
        kIOSurfaceHeight,
        kIOSurfaceBytesPerElement,
        kIOSurfacePixelFormat,
    };
    CFNumberRef nums[] = {
        CFNumberCreate(NULL, kCFNumberIntType, &width),
        CFNumberCreate(NULL, kCFNumberIntType, &height),
        CFNumberCreate(NULL, kCFNumberIntType, &four),
        CFNumberCreate(NULL, kCFNumberSInt32Type, &pix),
    };
    CFDictionaryRef props = CFDictionaryCreate(
        NULL, (const void **)keys, (const void **)nums, 4,
        &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks);
    IOSurfaceRef surf = IOSurfaceCreate(props);
    CFRelease(props);
    for (int i = 0; i < 4; i++) {
        CFRelease(nums[i]);
    }
    if (!surf) {
        fprintf(stderr, "IOSurfaceCreate failed\n");
        return 5;
    }
    uint32_t backing = IOSurfaceGetID(surf);
    fprintf(stderr, "create/import: backing_id=%u\n", backing);

    id<MTLDevice> device = pick_metal_device();
    int has_metal = device != nil;
    fprintf(stderr, "has_metal=%d name=%s\n", has_metal,
        device ? [[device name] UTF8String] : "nil");
    int metal_ok = 0;
    if (device) {
        MTLTextureDescriptor *td = [MTLTextureDescriptor
            texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
                                         width:(NSUInteger)width
                                        height:(NSUInteger)height
                                     mipmapped:NO];
        td.storageMode = MTLStorageModeShared;
        td.usage = MTLTextureUsageShaderRead | MTLTextureUsageRenderTarget;
        id<MTLTexture> tex = [device newTextureWithDescriptor:td iosurface:surf plane:0];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        if (tex && queue) {
            id<MTLCommandBuffer> buf = [queue commandBuffer];
            MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
            pass.colorAttachments[0].texture = tex;
            pass.colorAttachments[0].loadAction = MTLLoadActionClear;
            pass.colorAttachments[0].storeAction = MTLStoreActionStore;
            pass.colorAttachments[0].clearColor = MTLClearColorMake(0.95, 0.15, 0.85, 1.0);
            id<MTLRenderCommandEncoder> enc = [buf renderCommandEncoderWithDescriptor:pass];
            [enc endEncoding];
            [buf commit];
            [buf waitUntilCompleted];
            metal_ok = 1;
            fprintf(stderr, "metal wrap+clear ok iosurface_id=%u\n", backing);
        } else {
            fprintf(stderr, "metal wrap failed tex=%d queue=%d\n", tex != nil, queue != nil);
        }
    }
    if (!metal_ok) {
        IOSurfaceLock(surf, 0, NULL);
        void *base = IOSurfaceGetBaseAddress(surf);
        size_t bpr = IOSurfaceGetBytesPerRow(surf);
        for (int y = 0; y < height; y++) {
            uint32_t *row = (uint32_t *)((uint8_t *)base + (size_t)y * bpr);
            for (int x = 0; x < width; x++) {
                row[x] = 0xFFD21AE8u;
            }
        }
        IOSurfaceUnlock(surf, 0, NULL);
        fprintf(stderr, "cpu fill fallback (Metal encode unavailable)\n");
    }

    int token = 0;
    rc = begin(fb, &token);
    int rc_set = set_layer(fb, 0, surf, 0, 0, size[0], size[1], 0, 0, size[0], size[1], 0);
    int rc_end = end(fb);
    int rc_wait = wait(fb, token, 0);
    fprintf(stderr,
        "present: route=direct-iosurface copy=zero backing_id=%u "
        "has_metal=%d metal_ok=%d set=%d end=%d wait=%d token=%d\n",
        backing, has_metal, metal_ok, rc_set, rc_end, rc_wait, token);

    const double restore_w = size[0];
    const double restore_h = size[1];
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 20 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        IOSurfaceRef def = NULL;
        if (get_def) {
            (void)get_def(fb, 0, &def);
        }
        int t = 0;
        (void)begin(fb, &t);
        (void)set_layer(fb, 0, def, 0, 0, restore_w, restore_h, 0, 0, restore_w, restore_h, 0);
        (void)end(fb);
        (void)wait(fb, t, 0);
        CFRelease(surf);
        fprintf(stderr, "restore done\n");
    });
    return (rc == 0 && rc_set == 0 && rc_end == 0) ? 0 : 7;
}
