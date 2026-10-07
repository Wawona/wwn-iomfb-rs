/* Thin trampoline. Reconstructs CGRect packing for the C ABI. No Apple IOMFB header. */
#include <stdint.h>

typedef struct {
    double x;
    double y;
    double width;
    double height;
} iomfb_glue_rect;

extern int iomfb_swap_set_layer(
    void *display,
    int layer,
    void *surface,
    double sx,
    double sy,
    double sw,
    double sh,
    double dx,
    double dy,
    double dw,
    double dh,
    int flags);

int iomfb_swap_set_layer_rect(
    void *display,
    int layer,
    void *surface,
    iomfb_glue_rect src,
    iomfb_glue_rect dst,
    int flags)
{
    return iomfb_swap_set_layer(
        display,
        layer,
        surface,
        src.x,
        src.y,
        src.width,
        src.height,
        dst.x,
        dst.y,
        dst.width,
        dst.height,
        flags);
}
