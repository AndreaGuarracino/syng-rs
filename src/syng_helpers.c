/* Helper exports for syng-rs FFI.
 *
 * The syng C library uses some `static` globals and `static inline` helpers
 * in its headers. `static` at file scope has internal linkage in C, so the
 * symbols are NOT exported from the object file. We provide non-static
 * accessor functions here so Rust FFI can reach them.
 *
 * Compiled with -DONEIO by build.rs alongside the rest of vendor/syng.
 */

#include "syng.h"

/* `syngSchemaText` is `static char *` in syng.h. Each translation unit gets
 * its own copy with internal linkage; the symbol is not exported. Returning
 * its address from this non-static helper makes it reachable from Rust. */
const char *syng_rs_schema_text(void) {
    return syngSchemaText;
}
