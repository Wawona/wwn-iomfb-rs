/* Slim TrollStore smoke. Reconstructs the confirmed 26.1 swap family.
 * No Apple IOMFB header. dlopen the framework.
 */
#include <CoreFoundation/CoreFoundation.h>
#include <IOSurface/IOSurfaceRef.h>
#include <dlfcn.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
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

static const char *kPublicExports[] = {
#include "exports.inc"
};

static void *must_dlsym(void *h, const char *n) {
    void *p = dlsym(h, n);
    if (!p) {
        fprintf(stderr, "missing %s\n", n);
    }
    return p;
}

static void bind_census(void *lib) {
    size_t n = sizeof(kPublicExports) / sizeof(kPublicExports[0]);
    size_t ok = 0;
    for (size_t i = 0; i < n; i++) {
        if (dlsym(lib, kPublicExports[i])) {
            ok++;
        }
    }
    fprintf(stderr, "bind %zu/%zu\n", ok, n);
}

int main(void) {
    void *lib = dlopen(
        "/System/Library/PrivateFrameworks/IOMobileFramebuffer.framework/IOMobileFramebuffer",
        RTLD_LAZY);
    if (!lib) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 1;
    }
    FnGet get_main = must_dlsym(lib, "IOMobileFramebufferGetMainDisplay");
    FnGet get_sec = must_dlsym(lib, "IOMobileFramebufferGetSecondaryDisplay");
    FnSize get_size = must_dlsym(lib, "IOMobileFramebufferGetDisplaySize");
    FnBegin begin = must_dlsym(lib, "IOMobileFramebufferSwapBegin");
    FnEnd end = must_dlsym(lib, "IOMobileFramebufferSwapEnd");
    FnWait wait = must_dlsym(lib, "IOMobileFramebufferSwapWait");
    FnSetLayer set_layer = must_dlsym(lib, "IOMobileFramebufferSwapSetLayer");
    FnDefault get_def = must_dlsym(lib, "IOMobileFramebufferGetLayerDefaultSurface");
    FnPower power = must_dlsym(lib, "IOMobileFramebufferRequestPowerChange");
    bind_census(lib);
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
        (void)power(fb, 1);
    }

    CFStringRef keys[] = {
        kIOSurfaceWidth,
        kIOSurfaceHeight,
        kIOSurfaceBytesPerElement,
        kIOSurfacePixelFormat,
    };
    int32_t pix = 'BGRA';
    int four = 4;
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
    IOSurfaceLock(surf, 0, NULL);
    void *base = IOSurfaceGetBaseAddress(surf);
    size_t bpr = IOSurfaceGetBytesPerRow(surf);
    for (int y = 0; y < height; y++) {
        uint32_t *row = (uint32_t *)((uint8_t *)base + (size_t)y * bpr);
        for (int x = 0; x < width; x++) {
            row[x] = 0xFF2288CCu;
        }
    }
    IOSurfaceUnlock(surf, 0, NULL);

    int token = 0;
    rc = begin(fb, &token);
    if (rc != 0) {
        fprintf(stderr, "SwapBegin %d\n", rc);
        CFRelease(surf);
        return 6;
    }
    rc = set_layer(fb, 0, surf, 0, 0, size[0], size[1], 0, 0, size[0], size[1], 0);
    int rc_end = end(fb);
    int rc_wait = wait(fb, token, 0);
    fprintf(stderr, "present set=%d end=%d wait=%d token=%d\n", rc, rc_end, rc_wait, token);
    sleep(2);

    IOSurfaceRef def = NULL;
    if (get_def) {
        (void)get_def(fb, 0, &def);
    }
    token = 0;
    (void)begin(fb, &token);
    (void)set_layer(fb, 0, def, 0, 0, size[0], size[1], 0, 0, size[0], size[1], 0);
    (void)end(fb);
    (void)wait(fb, token, 0);
    fprintf(stderr, "restore done\n");
    CFRelease(surf);
    return (rc == 0 && rc_end == 0) ? 0 : 7;
}
