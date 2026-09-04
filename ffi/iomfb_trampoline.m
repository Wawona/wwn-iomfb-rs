/* Thin trampoline. Reconstructs CGRect packing for the C ABI.
 * No Apple IOMFB header. Calls iomfb_c only.
 */
#include <stdint.h>

#ifdef __OBJC__
#import <CoreGraphics/CoreGraphics.h>
#import <IOSurface/IOSurfaceRef.h>
#endif

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

#ifdef __OBJC__
int iomfb_swap_set_layer_rect(
    void *display,
    int layer,
    IOSurfaceRef surface,
    CGRect src,
    CGRect dst,
    int flags)
{
    return iomfb_swap_set_layer(
        display,
        layer,
        (void *)surface,
        src.origin.x,
        src.origin.y,
        src.size.width,
        src.size.height,
        dst.origin.x,
        dst.origin.y,
        dst.size.width,
        dst.size.height,
        flags);
}
#endif
