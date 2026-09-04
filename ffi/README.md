# ffi (ObjC trampoline)

Thin trampoline for `CGRect` / `IOSurface` only. Policy and the public API
live in Rust (`iomfb`, `iomfb-c`).

Do not grow product loops here. Do not include Apple IOMFB headers.
`dlopen` the framework at runtime.
