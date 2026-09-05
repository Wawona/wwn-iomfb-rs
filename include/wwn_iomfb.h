/* Wawona L4 Mode B sink ABI. Same names as the frozen wwn-iland-iomfb
 * header so Wawona can switch the link without rewriting ios_modeb.rs.
 * Implementation is this crate. L1 must not import it.
 */
#ifndef WWN_IOMFB_H_SINK
#define WWN_IOMFB_H_SINK

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint32_t x;
    uint32_t y;
    uint32_t width;
    uint32_t height;
} WwnIomfbDamage;

typedef struct {
    void *iosurface;
    uint32_t id;
    uint32_t width;
    uint32_t height;
    uint32_t bytes_per_row;
} WwnIomfbSurface;

int32_t wwn_iomfb_open(void **out_session);
int32_t wwn_iomfb_acquire(void *session, WwnIomfbSurface *out_surface);
int32_t wwn_iomfb_present_iosurface(
    void *session,
    void *iosurface,
    WwnIomfbDamage damage);
int32_t wwn_iomfb_present_metal_texture(
    void *session,
    void *metal_texture,
    WwnIomfbDamage damage);
int32_t wwn_iomfb_restore(void *session);
int32_t wwn_iomfb_set_exclusive(void *session, int32_t exclusive);
const char *wwn_iomfb_last_error(void *session);
void wwn_iomfb_destroy(void *session);

#ifdef __cplusplus
}
#endif

#endif
