/* Jailbreak CLI consumer. Same reconstructed ABI as include/iomfb.h.
 * Links nothing from Apple IOMFB headers. Push to /var/jb/usr/local/bin.
 */
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef void *Fb;
typedef int32_t (*FnGet)(Fb *);
typedef int32_t (*FnSize)(Fb, double *);
typedef int32_t (*FnU32)(Fb, uint32_t *);
typedef int32_t (*FnUnary)(Fb);

static const char *kPublicExports[] = {
#include "../tipa-smoke/exports.inc"
};

int main(void) {
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
    printf("bind %zu/%zu\n", bound, n);
    printf("stub SwapSignal=%p SwapSetUISubRegion=%p\n",
        dlsym(lib, "IOMobileFramebufferSwapSignal"),
        dlsym(lib, "IOMobileFramebufferSwapSetUISubRegion"));

    FnGet get_main = dlsym(lib, "IOMobileFramebufferGetMainDisplay");
    FnSize get_size = dlsym(lib, "IOMobileFramebufferGetDisplaySize");
    FnU32 get_id = dlsym(lib, "IOMobileFramebufferGetID");
    FnU32 is_main = dlsym(lib, "IOMobileFramebufferIsMainDisplay");
    FnUnary ready = dlsym(lib, "IOMobileFramebufferReadyForSwap");
    if (!get_main || !get_size) {
        return 2;
    }
    Fb fb = NULL;
    int32_t rc = get_main(&fb);
    if (rc != 0 || !fb) {
        fprintf(stderr, "GetMainDisplay %d\n", rc);
        return 3;
    }
    double size[2] = {0, 0};
    rc = get_size(fb, size);
    uint32_t id = 0;
    uint32_t main = 0;
    if (get_id) {
        (void)get_id(fb, &id);
    }
    if (is_main) {
        (void)is_main(fb, &main);
    }
    printf("display %.0fx%.0f id=%u main=%u ready=%p\n", size[0], size[1], id, main, (void *)ready);
    return bound == n ? 0 : 4;
}
