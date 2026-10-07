/* IOSurface glue. No Apple IOMFB header. Metal lives in iomfb_metal.swift. */
#include <CoreFoundation/CoreFoundation.h>
#include <IOSurface/IOSurfaceRef.h>
#include <stdint.h>

static int iomfb_glue_bytes_per_element(uint32_t fourcc) {
    switch (fourcc) {
        case 'BGRA':
            return 4;
        default:
            return 0;
    }
}

void *iomfb_glue_surface_create(uint32_t width, uint32_t height, uint32_t fourcc) {
    int bpp = iomfb_glue_bytes_per_element(fourcc);
    if (width == 0 || height == 0 || bpp == 0) {
        return NULL;
    }
    int32_t w = (int32_t)width;
    int32_t h = (int32_t)height;
    CFStringRef keys[] = {
        kIOSurfaceWidth,
        kIOSurfaceHeight,
        kIOSurfaceBytesPerElement,
        kIOSurfacePixelFormat,
    };
    CFNumberRef nums[] = {
        CFNumberCreate(NULL, kCFNumberSInt32Type, &w),
        CFNumberCreate(NULL, kCFNumberSInt32Type, &h),
        CFNumberCreate(NULL, kCFNumberIntType, &bpp),
        CFNumberCreate(NULL, kCFNumberSInt32Type, &fourcc),
    };
    CFDictionaryRef props = CFDictionaryCreate(
        NULL, (const void **)keys, (const void **)nums, 4,
        &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks);
    IOSurfaceRef surface = IOSurfaceCreate(props);
    CFRelease(props);
    for (int i = 0; i < 4; i++) {
        CFRelease(nums[i]);
    }
    return surface;
}

void iomfb_glue_surface_retain(void *surface) {
    if (surface) {
        CFRetain(surface);
    }
}

void iomfb_glue_surface_release(void *surface) {
    if (surface) {
        CFRelease(surface);
    }
}

uint32_t iomfb_glue_surface_id(void *surface) {
    if (!surface) {
        return 0;
    }
    return IOSurfaceGetID((IOSurfaceRef)surface);
}

uint32_t iomfb_glue_surface_width(void *surface) {
    if (!surface) {
        return 0;
    }
    return (uint32_t)IOSurfaceGetWidth((IOSurfaceRef)surface);
}

uint32_t iomfb_glue_surface_height(void *surface) {
    if (!surface) {
        return 0;
    }
    return (uint32_t)IOSurfaceGetHeight((IOSurfaceRef)surface);
}

uint32_t iomfb_glue_surface_bytes_per_row(void *surface) {
    if (!surface) {
        return 0;
    }
    return (uint32_t)IOSurfaceGetBytesPerRow((IOSurfaceRef)surface);
}

uint32_t iomfb_glue_surface_fourcc(void *surface) {
    if (!surface) {
        return 0;
    }
    return IOSurfaceGetPixelFormat((IOSurfaceRef)surface);
}

int iomfb_glue_surface_matches(void *surface, uint32_t width, uint32_t height, uint32_t fourcc) {
    if (!surface) {
        return 0;
    }
    IOSurfaceRef s = (IOSurfaceRef)surface;
    return IOSurfaceGetWidth(s) == width && IOSurfaceGetHeight(s) == height
        && IOSurfaceGetPixelFormat(s) == fourcc;
}
