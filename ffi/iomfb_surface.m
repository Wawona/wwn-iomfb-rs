/* IOSurface + Metal trampoline. No Apple IOMFB header.
 * Zero-copy: Metal texture is created from the IOSurface. IOMFB
 * receives that same IOSurface. Never blit here.
 */
#import <CoreFoundation/CoreFoundation.h>
#import <IOSurface/IOSurfaceRef.h>
#import <Metal/Metal.h>
#import <stdint.h>

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

void *iomfb_glue_metal_device(void) {
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    return (__bridge_retained void *)device;
}

void iomfb_glue_metal_release(void *object) {
    if (object) {
        CFRelease(object);
    }
}

/* Wrap the IOSurface as a Metal texture. Same backing. No blit. */
void *iomfb_glue_metal_texture_wrap(void *device, void *surface, int render_target) {
    if (!device || !surface) {
        return NULL;
    }
    IOSurfaceRef s = (IOSurfaceRef)surface;
    uint32_t fourcc = IOSurfaceGetPixelFormat(s);
    MTLPixelFormat fmt = MTLPixelFormatInvalid;
    if (fourcc == 'BGRA') {
        fmt = MTLPixelFormatBGRA8Unorm;
    }
    if (fmt == MTLPixelFormatInvalid) {
        return NULL;
    }
    NSUInteger w = IOSurfaceGetWidth(s);
    NSUInteger h = IOSurfaceGetHeight(s);
    MTLTextureDescriptor *td =
        [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:fmt
                                                           width:w
                                                          height:h
                                                       mipmapped:NO];
    td.storageMode = MTLStorageModeShared;
    td.usage = MTLTextureUsageShaderRead;
    if (render_target) {
        td.usage |= MTLTextureUsageRenderTarget;
    }
    id<MTLDevice> dev = (__bridge id<MTLDevice>)device;
    id<MTLTexture> tex = [dev newTextureWithDescriptor:td iosurface:s plane:0];
    return (__bridge_retained void *)tex;
}
