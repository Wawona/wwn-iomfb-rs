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

uint32_t iomfb_bound_export_count(void);

#ifdef __cplusplus
}
#endif

#endif
