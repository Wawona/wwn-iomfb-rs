/* Live matrix for every iomfb-c implementation. Links libiomfb_c.a.
 * Records return codes. Does not call KernelTests / HDCP send / FactoryPortal.
 */
#include "iomfb.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int g_fail;

static void rec(const char *name, int rc) {
    printf("rc %s %d\n", name, rc);
    if (rc != IOMFB_C_OK && rc != IOMFB_C_ABSENT && rc != 0xe00002c2 && rc != 0xe00002c7
        && rc != 0xe00002f0 && rc != 0xe00002bc) {
        /* Non-zero Apple IOReturn still counts as a live call. */
    }
    (void)g_fail;
}

int main(void) {
    setenv("WWN_IOMFB_APPLE", "1", 0);
    uint32_t bound = iomfb_bound_export_count();
    printf("bound %u/153\n", bound);
    printf("stub_signal %d stub_uisub %d\n",
        iomfb_export_is_stub("IOMobileFramebufferSwapSignal"),
        iomfb_export_is_stub("IOMobileFramebufferSwapSetUISubRegion"));
    printf("export_signal %p export_uisub %p export_main %p\n",
        iomfb_export("IOMobileFramebufferSwapSignal"),
        iomfb_export("IOMobileFramebufferSwapSetUISubRegion"),
        iomfb_export("IOMobileFramebufferGetMainDisplay"));

    void *fb = NULL;
    rec("display_open_main", iomfb_display_open_main(&fb));
    if (!fb) {
        printf("FAIL no display\n");
        return 2;
    }

    uint32_t w = 0, h = 0, id = 0, main = 0, cur = 0;
    rec("display_size", iomfb_display_size(fb, &w, &h));
    rec("display_id", iomfb_display_id(fb, &id));
    rec("display_is_main", iomfb_display_is_main(fb, &main));
    rec("swap_get_current", iomfb_swap_get_current(fb, &cur));
    printf("display %ux%u id=%u main=%u cur=%u\n", w, h, id, main, cur);

    rec("ready_via_call", iomfb_call_unary(fb, "IOMobileFramebufferWaitSurface"));
    rec("swap_cancel_all", iomfb_swap_cancel_all(fb));
    rec("call_cancel_all", iomfb_call_unary(fb, "IOMobileFramebufferSwapCancelAll"));
    rec("disable_vsync", iomfb_call_unary(fb, "IOMobileFramebufferDisableVSyncNotifications"));
    rec("disable_power", iomfb_call_unary(fb, "IOMobileFramebufferDisablePowerNotifications"));
    rec("disable_crc", iomfb_call_unary(fb, "IOMobileFramebufferDisableCRCNotifications"));
    rec("disable_hotplug", iomfb_call_unary(fb, "IOMobileFramebufferDisableHotPlugDetectNotifications"));
    rec("disable_needswap", iomfb_call_unary(fb, "IOMobileFramebufferDisableNeedSwapNotifications"));
    rec("stub_call", iomfb_call_unary(fb, "IOMobileFramebufferSwapSignal"));

    rec("power_savings_off", iomfb_call_int(fb, "IOMobileFramebufferEnableDisableVideoPowerSavings", 0));
    rec("request_power_on", iomfb_call_int(fb, "IOMobileFramebufferRequestPowerChange", 1));
    rec("set_droppable_0", iomfb_call_int(fb, "IOMobileFramebufferSetDroppable", 0));
    rec("set_droppable_1", iomfb_call_int(fb, "IOMobileFramebufferSetDroppable", 1));
    rec("brightness_0", iomfb_call_int(fb, "IOMobileFramebufferSetBrightnessCorrection", 0));
    rec("dither_1", iomfb_call_int(fb, "IOMobileFramebufferEnableDisableDithering", 1));

    int32_t remap = 0;
    rec("get_color_remap", iomfb_get_color_remap_mode(fb, &remap));
    rec("set_color_remap", iomfb_set_color_remap_mode(fb, remap));
    rec("white_on_black_0", iomfb_set_white_on_black(fb, 0));
    rec("factory_cal_begin", iomfb_factory_calibration_begin(fb));

    uint32_t out_u32 = 0;
    rec("call_ptr_id", iomfb_call_ptr(fb, "IOMobileFramebufferGetID", &out_u32));
    rec("call_ptr_main", iomfb_call_ptr(fb, "IOMobileFramebufferIsMainDisplay", &out_u32));
    rec("call_ptr_dotpitch", iomfb_call_ptr(fb, "IOMobileFramebufferGetDotPitch", &out_u32));
    rec("call_ptr_area", iomfb_call_ptr(fb, "IOMobileFramebufferGetDisplayArea", &out_u32));
    rec("call_ptr_time", iomfb_call_ptr(fb, "IOMobileFramebufferGetCurrentAbsoluteTime", &out_u32));
    rec("call_ptr_link", iomfb_call_ptr(fb, "IOMobileFramebufferGetLinkQuality", &out_u32));
    rec("call_ptr_hdcp_state", iomfb_call_ptr(fb, "IOMobileFramebufferGetHDCPDownstreamState", &out_u32));
    rec("call_ptr_protect", iomfb_call_ptr(fb, "IOMobileFramebufferGetProtectionOptions", &out_u32));
    rec("call_ptr_digital", iomfb_call_ptr(fb, "IOMobileFramebufferGetDigitalOutState", &out_u32));
    rec("call_ptr_mirror", iomfb_call_ptr(fb, "IOMobileFramebufferGetMirrorError", &out_u32));
    rec("call_ptr_pwm", iomfb_call_ptr(fb, "IOMobileFramebufferGetPulseWidthMaximization", &out_u32));
    rec("call_ptr_cancel_cur", iomfb_call_ptr(fb, "IOMobileFramebufferSwapCancelAllGetCurrent", &out_u32));
    rec("wrong_arity", iomfb_call_unary(fb, "IOMobileFramebufferSwapBegin"));

    uint32_t tx = 0, ty = 0;
    rec("touch_map", iomfb_touch_map(fb, 0.5, 0.5, &tx, &ty));
    printf("touch_map 0.5,0.5 -> %u,%u\n", tx, ty);

    void *seat = NULL;
    rec("touch_open", iomfb_touch_open(fb, &seat));
    if (seat) {
        rec("touch_set_view", iomfb_touch_set_view(seat, (double)w, (double)h));
        rec("touch_set_dest", iomfb_touch_set_dest(seat, 0, 0, (double)w, (double)h));
        rec("touch_set_rot", iomfb_touch_set_rotation(seat, 0));
        iomfb_touch_event ev;
        memset(&ev, 0, sizeof(ev));
        rec("touch_down", iomfb_touch_inject(seat, 1, IOMFB_TOUCH_DOWN, 0.25, 0.25, IOMFB_TOUCH_SPACE_NORMALIZED, &ev));
        printf("touch_ev id=%d state=%d xy=%u,%u slot=%u\n", ev.id, ev.state, ev.x, ev.y, ev.slot);
        rec("touch_move", iomfb_touch_inject(seat, 1, IOMFB_TOUCH_MOTION, 0.30, 0.30, IOMFB_TOUCH_SPACE_NORMALIZED, &ev));
        rec("touch_up", iomfb_touch_inject(seat, 1, IOMFB_TOUCH_UP, 0.30, 0.30, IOMFB_TOUCH_SPACE_NORMALIZED, &ev));
        rec("touch_pixel", iomfb_touch_inject(seat, 2, IOMFB_TOUCH_DOWN, 10, 20, IOMFB_TOUCH_SPACE_PIXEL, &ev));
        rec("touch_hid", iomfb_touch_inject(seat, 2, IOMFB_TOUCH_UP, 10, 20, IOMFB_TOUCH_SPACE_HID, &ev));
        rec("touch_view", iomfb_touch_inject(seat, 3, IOMFB_TOUCH_DOWN, 100, 200, IOMFB_TOUCH_SPACE_VIEW, &ev));
        rec("touch_cancel_one", iomfb_touch_inject(seat, 3, IOMFB_TOUCH_CANCEL, 100, 200, IOMFB_TOUCH_SPACE_VIEW, &ev));
        printf("touch_active %d\n", iomfb_touch_active_count(seat));
        rec("touch_cancel_all", iomfb_touch_cancel_all(seat));
        iomfb_touch_close(seat);
    }

    void *sc = NULL;
    rec("swapchain_open", iomfb_swapchain_open(&sc));
    if (sc) {
        uint32_t sw = 0, sh = 0, sid = 0;
        void *surf = NULL;
        void *mtl = NULL;
        rec("swapchain_size", iomfb_swapchain_size(sc, &sw, &sh));
        int has_metal = iomfb_swapchain_has_metal(sc);
        printf("swapchain %ux%u has_metal=%d\n", sw, sh, has_metal);
        rec("swapchain_acquire", iomfb_swapchain_acquire(sc, &surf, &mtl, &sid));
        rec("swapchain_clear", iomfb_swapchain_clear(sc, 0.2f, 0.0f, 0.4f, 1.0f));
        iomfb_present_info info;
        memset(&info, 0, sizeof(info));
        rec("swapchain_present", iomfb_swapchain_present(sc, &info));
        printf("present token=%d wait=%d displayed=%u zero_copy=%u sid=%u\n",
            info.token, info.wait_rc, info.displayed, info.zero_copy, sid);
        if (surf) {
            memset(&info, 0, sizeof(info));
            rec("present_external", iomfb_swapchain_present_external(sc, surf, &info));
            rec("present_iosurface", iomfb_present_iosurface(fb, surf, &info));
        }
        void *seat2 = NULL;
        rec("touch_open_sc", iomfb_touch_open_swapchain(sc, &seat2));
        if (seat2) {
            iomfb_touch_close(seat2);
        }
        rec("restore", iomfb_restore_default_surface(fb));
        iomfb_swapchain_close(sc);
    } else {
        int32_t token = 0;
        rec("swap_begin", iomfb_swap_begin(fb, &token));
        rec("swap_cancel", iomfb_swap_cancel(fb, token));
        rec("restore", iomfb_restore_default_surface(fb));
    }

    iomfb_display_close(fb);
    printf("matrix done bound=%u\n", bound);
    return bound == 153 ? 0 : 4;
}
