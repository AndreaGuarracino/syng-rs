/* Helper exports for syng-rs FFI.
 *
 * The syng C library uses some `static` globals and `static inline` helpers
 * in its headers. `static` at file scope has internal linkage in C, so the
 * symbols are NOT exported from the object file. We provide non-static
 * accessor functions here so Rust FFI can reach them.
 *
 * Compiled with -DONEIO by build.rs alongside the rest of vendor/syng.
 */

/* syng.h transitively includes kmerhash.h via syncmerset.h. seqhash.h
 * has Seqhash + SeqhashIterator types we need for the destroy helpers. */
#include "syng.h"
#include "seqhash.h"

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

/* Per-path metadata accessors.
 *
 * sb->path is an Array of SyngPath { U32 file; U32 path; U64 length; }
 * (defined in syng.h). The Array is opaque to Rust; arrp() / arr() macros
 * yield pointers/values from the array. We expose each field individually so
 * Rust doesn't have to know the struct layout.
 *
 * path_idx is 0-based (matches C convention) and MUST be < arrayMax(sb->path).
 * No bounds checking here; callers must validate.
 */
U64 syng_rs_gbwt_path_length(const SyngBWT *sb, I64 path_idx) {
    return arrp(sb->path, (U64)path_idx, SyngPath)->length;
}

U32 syng_rs_gbwt_path_file(const SyngBWT *sb, I64 path_idx) {
    return arrp(sb->path, (U64)path_idx, SyngPath)->file;
}

U32 syng_rs_gbwt_path_id(const SyngBWT *sb, I64 path_idx) {
    return arrp(sb->path, (U64)path_idx, SyngPath)->path;
}

/* SyngBWTpath cursor accessors. The struct is publicly defined in syng.h
 * but Rust treats it as opaque (we don't want layout coupling). These let
 * sidecar builders snapshot the GBWT cursor state at sample points so the
 * cursor can be re-entered later via syngBWTpathStartOld(thisNode, jLast). */
U32 syng_rs_path_jlast(const SyngBWTpath *sbp) {
    return sbp->jLast;
}

I32 syng_rs_path_this_node(const SyngBWTpath *sbp) {
    return sbp->thisNode;
}

I32 syng_rs_path_last_node(const SyngBWTpath *sbp) {
    return sbp->lastNode;
}

U32 syng_rs_path_last_off(const SyngBWTpath *sbp) {
    return sbp->lastOff;
}

/* `seqhashDestroy` and `seqhashIteratorDestroy` are `static` in
 * seqhash.h (lines 38, 51), so they have internal linkage and Rust FFI
 * cannot call them across the .o boundary. Re-export through this TU
 * which can see the inline definitions. */
void syng_rs_seqhash_destroy(Seqhash *sh) {
    seqhashDestroy(sh);
}

void syng_rs_seqhash_iterator_destroy(SeqhashIterator *si) {
    seqhashIteratorDestroy(si);
}

/* ─── ONElib field accessors ─────────────────────────────────────────
 *
 * The oneInt(), oneLen(), oneString(), oneIntList(), oneDNAchar()
 * "accessors" in ONElib.h are CPP macros that index into the OneFile
 * struct's `field[]` array. Macros can't cross the FFI boundary; expose
 * them as real functions so Rust callers can read field values after
 * each oneReadLine.
 */

I64 syng_rs_one_int(OneFile *of, int x) {
    return oneInt(of, x);
}

I64 syng_rs_one_len(OneFile *of) {
    return oneLen(of);
}

const char *syng_rs_one_string(OneFile *of) {
    return oneString(of);
}

const I64 *syng_rs_one_int_list(OneFile *of) {
    return oneIntList(of);
}

const char *syng_rs_one_dna_char(OneFile *of) {
    return oneDNAchar(of);
}
