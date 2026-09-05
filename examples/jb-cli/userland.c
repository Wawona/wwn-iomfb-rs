/* Product userspace smoke. Never sets WWN_IOMFB_APPLE.
 * Built without apple-iomfb so this binary cannot dlopen Apple IOMFB.
 */
#include "../../include/iomfb.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int g_presents;

static void on_present(
    void *ctx,
    void *surface,
    uint32_t width,
    uint32_t height,
    int32_t token,
    int32_t layer)
{
    int *hits = ctx;
    if (hits) {
        (*hits)++;
    }
    printf(
        "present surface=%p %ux%u token=%d layer=%d\n",
        surface,
        width,
        height,
        token,
        layer);
}

static int fail(const char *what, int rc)
{
    fprintf(stderr, "FAIL %s rc=%d\n", what, rc);
    return 1;
}

int main(void)
{
    unsetenv("WWN_IOMFB_APPLE");
    unsetenv("WWN_IOMFB_CHANNEL");
    iomfb_channel_set(IOMFB_CHANNEL_TROLLSTORE);
    if (iomfb_full_re() != 0) {
        return fail("trollstore must not be full RE", iomfb_full_re());
    }

    if (iomfb_bound_export_count() != 0) {
        return fail("bound_export_count must be 0 without Apple", (int)iomfb_bound_export_count());
    }

    int rc = iomfb_display_configure(640, 480);
    if (rc != IOMFB_C_OK) {
        return fail("configure", rc);
    }

    void *fb = NULL;
    rc = iomfb_display_open_userland(640, 480, &fb);
    if (rc != IOMFB_C_OK || !fb) {
        return fail("open_userland", rc);
    }
    if (iomfb_display_is_userland(fb) != 1) {
        return fail("is_userland", iomfb_display_is_userland(fb));
    }

    uint32_t w = 0;
    uint32_t h = 0;
    rc = iomfb_display_size(fb, &w, &h);
    if (rc != IOMFB_C_OK || w != 640 || h != 480) {
        return fail("size", rc);
    }

    if (iomfb_factory_calibration_begin(fb) != IOMFB_C_ABSENT) {
        return fail("factory must be Absent", iomfb_factory_calibration_begin(fb));
    }
    if (iomfb_factory_portal(fb, NULL) != IOMFB_C_ABSENT) {
        return fail("factory_portal", iomfb_factory_portal(fb, NULL));
    }
    if (iomfb_kernel_tests(fb, NULL) != IOMFB_C_ABSENT) {
        return fail("kernel_tests", iomfb_kernel_tests(fb, NULL));
    }
    if (iomfb_hdcp_send_request(fb, NULL, 0, NULL, 0) != IOMFB_C_ABSENT) {
        return fail("hdcp", iomfb_hdcp_send_request(fb, NULL, 0, NULL, 0));
    }

    int32_t live_rc = 0;
    if (iomfb_live_call(fb, "IOMobileFramebufferSwapEnd", &live_rc) != IOMFB_C_ABSENT) {
        return fail("live_call", live_rc);
    }

    iomfb_display_set_present(fb, on_present, &g_presents);
    int32_t token = 0;
    rc = iomfb_swap_begin(fb, &token);
    if (rc != IOMFB_C_OK || token <= 0) {
        return fail("swap_begin", rc);
    }
    rc = iomfb_swap_set_layer(fb, 0, NULL, 0, 0, 640, 480, 0, 0, 640, 480, 0);
    if (rc != IOMFB_C_OK) {
        return fail("swap_set_layer", rc);
    }
    rc = iomfb_swap_end(fb);
    if (rc != IOMFB_C_OK) {
        return fail("swap_end", rc);
    }
    rc = iomfb_swap_wait(fb, token, 0);
    if (rc != IOMFB_C_OK) {
        return fail("swap_wait", rc);
    }
    if (g_presents < 1) {
        return fail("present callback", g_presents);
    }

    void *sw = NULL;
    rc = iomfb_swapchain_open(&sw);
    if (rc != IOMFB_C_OK || !sw) {
        return fail("swapchain_open", rc);
    }
    if (iomfb_swapchain_is_userland(sw) != 1) {
        return fail("swapchain_is_userland", iomfb_swapchain_is_userland(sw));
    }

    int presents = 0;
    iomfb_swapchain_set_present(sw, on_present, &presents);
    void *surf = NULL;
    void *metal = NULL;
    uint32_t sid = 0;
    rc = iomfb_swapchain_acquire(sw, &surf, &metal, &sid);
    if (rc != IOMFB_C_OK || !surf || sid == 0) {
        return fail("acquire", rc);
    }
    printf("acquire surface=%p metal=%p sid=%u has_metal=%d\n",
        surf, metal, sid, iomfb_swapchain_has_metal(sw));

    iomfb_present_info info;
    memset(&info, 0, sizeof(info));
    rc = iomfb_swapchain_present(sw, &info);
    if (rc != IOMFB_C_OK) {
        return fail("present", rc);
    }
    if (!info.zero_copy) {
        return fail("zero_copy", info.zero_copy);
    }
    if (presents < 1) {
        return fail("swapchain present callback", presents);
    }
    printf(
        "userland=1 bound=%u has_metal=%d sid=%u zero=%u token=%d wait=%d presents=%d\n",
        iomfb_bound_export_count(),
        iomfb_swapchain_has_metal(sw),
        sid,
        (unsigned)info.zero_copy,
        info.token,
        info.wait_rc,
        presents);

    iomfb_swapchain_close(sw);
    iomfb_display_close(fb);
    printf("PASS trollstore channel=%d full_re=0 no-IOConnect\n", iomfb_channel_get());
    return 0;
}
