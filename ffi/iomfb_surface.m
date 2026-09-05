/* IOSurface + Metal trampoline. No Apple IOMFB header.
 * Zero-copy: Metal texture is created from the IOSurface. The
 * userspace backend presents that same IOSurface. Never blit here.
 */
#import <CoreFoundation/CoreFoundation.h>
#import <IOSurface/IOSurfaceRef.h>
#import <Metal/Metal.h>
#import <stdint.h>
#import <TargetConditionals.h>
#if TARGET_OS_IPHONE
#import <UIKit/UIKit.h>
#endif

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

void *iomfb_glue_metal_queue(void *device) {
    if (!device) {
        return NULL;
    }
    id<MTLDevice> dev = (__bridge id<MTLDevice>)device;
    return (__bridge_retained void *)[dev newCommandQueue];
}

int iomfb_glue_metal_wait(void *queue) {
    if (!queue) {
        return -1;
    }
    id<MTLCommandQueue> q = (__bridge id<MTLCommandQueue>)queue;
    id<MTLCommandBuffer> buf = [q commandBuffer];
    if (!buf) {
        return -1;
    }
    [buf commit];
    [buf waitUntilCompleted];
    return 0;
}

/* GPU clear into the IOSurface-backed texture. Same backing. No blit. */
int iomfb_glue_metal_clear(
    void *queue, void *texture, float r, float g, float b, float a) {
    if (!queue || !texture) {
        return -1;
    }
    id<MTLCommandQueue> q = (__bridge id<MTLCommandQueue>)queue;
    id<MTLTexture> tex = (__bridge id<MTLTexture>)texture;
    id<MTLCommandBuffer> buf = [q commandBuffer];
    if (!buf) {
        return -1;
    }
    MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
    pass.colorAttachments[0].texture = tex;
    pass.colorAttachments[0].loadAction = MTLLoadActionClear;
    pass.colorAttachments[0].storeAction = MTLStoreActionStore;
    pass.colorAttachments[0].clearColor = MTLClearColorMake(r, g, b, a);
    id<MTLRenderCommandEncoder> enc = [buf renderCommandEncoderWithDescriptor:pass];
    if (!enc) {
        return -1;
    }
    [enc endEncoding];
    [buf commit];
    [buf waitUntilCompleted];
    return 0;
}

/* Public UIKit screen size. Not IOMFB. Not IOConnect. */
int iomfb_glue_host_size(uint32_t *width, uint32_t *height) {
#if TARGET_OS_IPHONE
    CGRect b = [UIScreen mainScreen].nativeBounds;
    uint32_t w = (uint32_t)b.size.width;
    uint32_t h = (uint32_t)b.size.height;
    if (w == 0 || h == 0) {
        return -1;
    }
    if (width) {
        *width = w;
    }
    if (height) {
        *height = h;
    }
    return 0;
#else
    (void)width;
    (void)height;
    return -1;
#endif
}
