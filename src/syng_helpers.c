/* Helper exports for syng-rs FFI.
 *
 * The syng C library uses some `static` globals and `static inline` helpers
 * in its headers. `static` at file scope has internal linkage in C, so the
 * symbols are NOT exported from the object file. We provide non-static
 * accessor functions here so Rust FFI can reach them.
 *
 * Compiled with -DONEIO by build.rs alongside the rest of vendor/syng.
 */

/* syng.h transitively includes kmerhash.h via syncmerset.h. */
#include "syng.h"

/* `syngSchemaText` is `static char *` in syng.h. Each translation unit gets
 * its own copy with internal linkage; the symbol is not exported. Returning
 * its address from this non-static helper makes it reachable from Rust. */
const char *syng_rs_schema_text(void) {
    return syngSchemaText;
}

/* `kmerHashMax(kh)` is a CPP macro in kmerhash.h; macros cannot cross the
 * FFI boundary. Re-export as a real function so Rust can call it. */
I64 syng_rs_kmer_hash_max(const KmerHash *kh) {
    return kh->max;
}

/* `kmerHashLen(kh)` is the syncmer length in bases (len+w as syncmer length).
 * Exposed for callers that need it to size DNA buffers. */
int syng_rs_kmer_hash_len(const KmerHash *kh) {
    return kh->len;
}

/* SyngBWT struct (syng.h): node/path fields are Array (opaque to Rust).
 * Expose their `max` counts via these accessors. arrayMax(ar) is a CPP
 * macro in array.h that returns (ar)->max. */
I64 syng_rs_gbwt_node_count(const SyngBWT *sb) {
    return arrayMax(sb->node);
}

I64 syng_rs_gbwt_path_count(const SyngBWT *sb) {
    return arrayMax(sb->path);
}

/* Fixed syncmer length stored on the GBWT (sb->fixedLen).
 * 0 means variable-length syncmers (use sb->length array per node). */
int syng_rs_gbwt_fixed_len(const SyngBWT *sb) {
    return sb->fixedLen;
}
