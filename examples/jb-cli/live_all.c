/* Record a return code for every public IOMobileFramebuffer* export.
 * Links libiomfb_c.a. Default is in-process (fork cannot reopen IOMFB).
 * `iomfb-live --one NAME` execs one export for a crashed row.
 */
#include "iomfb.h"

#include <CoreFoundation/CoreFoundation.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int run_one(const char *name, void *fb) {
    if (strcmp(name, "IOMobileFramebufferCopyProperty") == 0) {
        typedef void *(*CopyPropFn)(void *, const void *);
        CopyPropFn f = (CopyPropFn)iomfb_export(name);
        if (!f) {
            printf("live %s missing\n", name);
            return 5;
        }
        void *cf = f(fb, CFSTR("IOMFB"));
        printf("live %s rc=%d ptr=%p\n", name, cf ? 0 : 0, cf);
        return 0;
    }
    int32_t rc = 0;
    int call = iomfb_live_call(fb, name, &rc);
    if (call == IOMFB_C_ABSENT || rc == IOMFB_C_ABSENT) {
        printf("live %s absent\n", name);
        return 4;
    }
    if (call == IOMFB_C_MISSING || call == IOMFB_C_LOAD) {
        printf("live %s missing call=%d\n", name, call);
        return 5;
    }
    printf("live %s rc=%d\n", name, rc);
    return 0;
}

static void typed_helpers(void *fb) {
    uintptr_t type_id = 0;
    int tid_rc = iomfb_get_type_id(&type_id);
    printf("typed type_id call=%d id=%lu\n", tid_rc, (unsigned long)type_id);
    printf("typed service %p\n", iomfb_get_service_object(fb));
    printf("typed ready %d\n", iomfb_ready_for_swap(fb));
    printf("typed enable_vsync %d\n", iomfb_enable_vsync(fb));
    printf("typed disable_vsync %d\n", iomfb_disable_vsync(fb));

    uint8_t gamma[IOMFB_GAMMA_TABLE_SIZE];
    memset(gamma, 0, sizeof(gamma));
    printf("typed get_gamma %d\n", iomfb_get_gamma_table(fb, gamma, sizeof(gamma)));
    printf("typed set_gamma %d\n", iomfb_set_gamma_table(fb, gamma, sizeof(gamma)));

    uint8_t kargs[IOMFB_KERNEL_TESTS_SIZE];
    memset(kargs, 0, sizeof(kargs));
    printf("typed kernel_tests %d\n", iomfb_kernel_tests(fb, kargs));
    printf("typed factory_portal %d\n", iomfb_factory_portal(fb, NULL));
    printf("typed hdcp_send %d\n", iomfb_hdcp_send_request(fb, NULL, 0, NULL, 0));

    void *by_name = NULL;
    int name_rc = iomfb_open_by_name((void *)CFSTR("primary"), &by_name);
    printf("typed open_by_name_primary %d ptr=%p\n", name_rc, by_name);
    if (by_name) {
        iomfb_display_close(by_name);
    }
}

int main(int argc, char **argv) {
    iomfb_channel_set(IOMFB_CHANNEL_JAILBREAK);
    setenv("WWN_IOMFB_APPLE", "1", 0);
    setenv("WWN_IOMFB_CHANNEL", "jailbreak", 0);
    setvbuf(stdout, NULL, _IONBF, 0);
    uint32_t n = iomfb_public_export_count();
    uint32_t bound = iomfb_bound_export_count();
    (void)bound;

    if (argc == 2 && strcmp(argv[1], "--probe") == 0) {
        printf("channel=%d full_re=%d bound=%u/153\n",
            iomfb_channel_get(), iomfb_full_re(), bound);
        void *probe = NULL;
        int open_rc = iomfb_display_open_jailbreak(&probe);
        printf("open_jailbreak rc=%d userland=%d jailbreak=%d\n",
            open_rc,
            probe ? iomfb_display_is_userland(probe) : -1,
            probe ? iomfb_display_is_jailbreak(probe) : -1);
        if (probe) {
            uint32_t w = 0, h = 0;
            (void)iomfb_display_size(probe, &w, &h);
            printf("display %ux%u\n", w, h);
            iomfb_display_close(probe);
        }
        return (iomfb_full_re() == 1 && bound == 153 && open_rc == IOMFB_C_OK) ? 0 : 2;
    }

    if (argc == 2 && strcmp(argv[1], "--list") == 0) {
        for (uint32_t i = 0; i < n; i++) {
            const char *name = iomfb_public_export_name(i);
            if (name) {
                printf("%s\n", name);
            }
        }
        return 0;
    }

    void *fb = NULL;
    int open_rc = iomfb_display_open_main(&fb);
    if (!fb) {
        printf("FAIL no display open=%d\n", open_rc);
        return 2;
    }

    if (argc == 3 && strcmp(argv[1], "--one") == 0) {
        int rc = run_one(argv[2], fb);
        iomfb_display_close(fb);
        return rc;
    }

    printf("live_all exports=%u bound=%u/153\n", n, bound);
    printf("display_open_main %d\n", open_rc);

    uint32_t w = 0, h = 0, id = 0, maind = 0;
    (void)iomfb_display_size(fb, &w, &h);
    (void)iomfb_display_id(fb, &id);
    (void)iomfb_display_is_main(fb, &maind);
    printf("display %ux%u id=%u main=%u\n", w, h, id, maind);
    typed_helpers(fb);

    uint32_t called = 0;
    uint32_t absent = 0;
    uint32_t missing = 0;
    for (uint32_t i = 0; i < n; i++) {
        const char *name = iomfb_public_export_name(i);
        if (!name) {
            printf("FAIL missing name %u\n", i);
            iomfb_display_close(fb);
            return 3;
        }
        int rc = run_one(name, fb);
        if (rc == 4) {
            absent++;
        } else if (rc == 0) {
            called++;
        } else {
            missing++;
        }
    }

    iomfb_display_close(fb);
    printf("live_all done called=%u absent=%u missing=%u exports=%u bound=%u\n",
        called, absent, missing, n, bound);
    return (n == 153 && called + absent == 153 && missing == 0) ? 0 : 4;
}
