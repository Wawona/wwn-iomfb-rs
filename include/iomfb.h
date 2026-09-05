/* Reconstructed IOMFB C ABI. Not an Apple header.
 *
 * TrollStore tipas and jailbreak tweaks link this. Wawona Mode B
 * Desktop will call the same present path.
 *
 * GPU present is zero-copy: draw into an IOSurface-backed Metal
 * texture, then iomfb_swapchain_present. Wawona compositors pass
 * their IOSurface to iomfb_swapchain_present_external.
 */
#ifndef WWN_IOMFB_H
#define WWN_IOMFB_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define IOMFB_C_OK 0
#define IOMFB_C_UNCONFIRMED -1
#define IOMFB_C_MISSING -2
#define IOMFB_C_LOAD -3
#define IOMFB_C_ABSENT -4
#define IOMFB_C_SURFACE -5
#define IOMFB_C_NULL_SURFACE -6
#define IOMFB_C_INCOMPATIBLE -7

#define IOMFB_GAMMA_TABLE_SIZE 0xc0c
#define IOMFB_KERNEL_TESTS_SIZE 0x9c
#define IOMFB_PUBLIC_EXPORTS 153

#define IOMFB_TOUCH_UP 0
#define IOMFB_TOUCH_DOWN 1
#define IOMFB_TOUCH_MOTION 2
#define IOMFB_TOUCH_CANCEL 3
#define IOMFB_TOUCH_SLOTS 16
#define IOMFB_TOUCH_SPACE_NORMALIZED 0
#define IOMFB_TOUCH_SPACE_VIEW 1
#define IOMFB_TOUCH_SPACE_PIXEL 2
#define IOMFB_TOUCH_SPACE_HID 3

typedef struct iomfb_touch_event {
    int32_t id;
    int32_t state;
    uint32_t x;
    uint32_t y;
    double nx;
    double ny;
    uint8_t slot;
} iomfb_touch_event;

typedef struct iomfb_present_info {
    int32_t token;
    int32_t wait_rc;
    uint8_t displayed;
    uint8_t zero_copy;
} iomfb_present_info;

int iomfb_display_open_main(void **out);
int iomfb_display_size(void *display, uint32_t *w, uint32_t *h);
int iomfb_swap_begin(void *display, int32_t *token);
int iomfb_swap_set_layer(
    void *display,
    int32_t layer,
    void *surface,
    double sx,
    double sy,
    double sw,
    double sh,
    double dx,
    double dy,
    double dw,
    double dh,
    int32_t flags);
int iomfb_swap_end(void *display);
int iomfb_swap_wait(void *display, int32_t token, int32_t options);
int iomfb_swap_cancel(void *display, int32_t token);
int iomfb_restore_default_surface(void *display);
int iomfb_present_iosurface(void *display, void *surface, iomfb_present_info *out);
void iomfb_display_close(void *display);

int iomfb_swapchain_open(void **out);
int iomfb_swapchain_size(void *swapchain, uint32_t *w, uint32_t *h);
int iomfb_swapchain_has_metal(void *swapchain);
int iomfb_swapchain_acquire(
    void *swapchain,
    void **out_surface,
    void **out_metal,
    uint32_t *out_id);
int iomfb_swapchain_present(void *swapchain, iomfb_present_info *out);
int iomfb_swapchain_present_external(
    void *swapchain,
    void *surface,
    iomfb_present_info *out);
int iomfb_swapchain_clear(void *swapchain, float r, float g, float b, float a);
void iomfb_swapchain_close(void *swapchain);
int iomfb_display_id(void *display, uint32_t *out);
int iomfb_display_is_main(void *display, uint32_t *out);
int iomfb_swap_cancel_all(void *display);
int iomfb_swap_get_current(void *display, uint32_t *out);

int iomfb_touch_map(
    void *display,
    double nx,
    double ny,
    uint32_t *out_x,
    uint32_t *out_y);
int iomfb_touch_open(void *display, void **out);
int iomfb_touch_open_swapchain(void *swapchain, void **out);
int iomfb_touch_set_view(void *touch, double view_w, double view_h);
int iomfb_touch_set_dest(void *touch, double x, double y, double w, double h);
int iomfb_touch_set_rotation(void *touch, int32_t degrees);
int iomfb_touch_inject(
    void *touch,
    int32_t id,
    int32_t state,
    double x,
    double y,
    int32_t space,
    iomfb_touch_event *out);
int iomfb_touch_active_count(void *touch);
int iomfb_touch_cancel_all(void *touch);
void iomfb_touch_close(void *touch);

uint32_t iomfb_bound_export_count(void);
void *iomfb_export(const char *name);
int iomfb_export_is_stub(const char *name);
int iomfb_call_unary(void *display, const char *name);
int iomfb_call_int(void *display, const char *name, int32_t value);
int iomfb_call_ptr(void *display, const char *name, void *ptr);
int iomfb_set_white_on_black(void *display, int on);
int iomfb_set_color_remap_mode(void *display, int32_t mode);
int iomfb_get_color_remap_mode(void *display, int32_t *out);
int iomfb_factory_calibration_begin(void *display);
int iomfb_factory_portal(void *display, void *arg);
int iomfb_kernel_tests(void *display, void *args);
int iomfb_hdcp_send_request(
    void *display,
    void *req,
    uint32_t req_len,
    void *reply,
    uint32_t reply_len);

int iomfb_get_gamma_table(void *display, void *buf, uint32_t len);
int iomfb_set_gamma_table(void *display, const void *buf, uint32_t len);
int iomfb_enable_vsync(void *display);
int iomfb_disable_vsync(void *display);
int iomfb_get_type_id(uintptr_t *out);
void *iomfb_get_service_object(void *display);
int iomfb_open_by_name(void *cf_name, void **out);
int iomfb_ready_for_swap(void *display);

uint32_t iomfb_public_export_count(void);
const char *iomfb_public_export_name(uint32_t i);
int iomfb_live_call(void *display, const char *name, int32_t *out_rc);

#ifdef __cplusplus
}
#endif

#endif
