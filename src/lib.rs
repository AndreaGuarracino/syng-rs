//! Raw FFI bindings to the syng C library.
//!
//! Layered: this crate exposes `extern "C"` declarations matching syng's C
//! headers verbatim. Types are opaque (`repr(C)` with `_private: [u8; 0]`)
//! where Rust does not need to inspect them; pointer ownership is the
//! caller's problem.
//!
//! Safe RAII wrappers live in higher crates (upang). This crate is FFI-only.
//!
//! C library version pinned via the `vendor/syng` git submodule.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use libc::{c_char, c_int, FILE};

// Type aliases matching syng's utils.h.
pub type I8 = i8;
pub type I16 = i16;
pub type I32 = i32;
pub type I64 = i64;
pub type U8 = u8;
pub type U16 = u16;
pub type U32 = u32;
pub type U64 = u64;

// ─── Opaque handles ──────────────────────────────────────────────────────────

#[repr(C)]
pub struct Seqhash {
    _private: [u8; 0],
}
#[repr(C)]
pub struct SeqhashIterator {
    _private: [u8; 0],
}
#[repr(C)]
pub struct KmerHash {
    _private: [u8; 0],
}
#[repr(C)]
pub struct SyngBWT {
    _private: [u8; 0],
}
#[repr(C)]
pub struct SyngBWTpath {
    _private: [u8; 0],
}
#[repr(C)]
pub struct OneFile {
    _private: [u8; 0],
}
#[repr(C)]
pub struct OneSchema {
    _private: [u8; 0],
}

// ─── seqhash.h ────────────────────────────────────────────────────────────────

extern "C" {
    pub fn seqhashCreate(k: c_int, w: c_int, seed: c_int) -> *mut Seqhash;
    pub fn seqhashWrite(sh: *mut Seqhash, f: *mut FILE);
    pub fn seqhashRead(f: *mut FILE) -> *mut Seqhash;

    pub fn syncmerIterator(sh: *mut Seqhash, s: *mut c_char, len: c_int) -> *mut SeqhashIterator;
    pub fn syncmerNext(
        si: *mut SeqhashIterator,
        kmer: *mut U64,
        pos: *mut c_int,
        isF: *mut bool,
    ) -> bool;
}

// ─── kmerhash.h ───────────────────────────────────────────────────────────────

extern "C" {
    pub fn kmerHashCreate(initial_size: U64, len: c_int) -> *mut KmerHash;
    pub fn kmerHashDestroy(kh: *mut KmerHash);

    pub fn kmerHashAdd(kh: *mut KmerHash, dna: *mut c_char, index: *mut I64) -> bool;
    pub fn kmerHashFind(kh: *mut KmerHash, dna: *mut c_char, index: *mut I64) -> bool;
    pub fn kmerHashFindThreadSafe(
        kh: *mut KmerHash,
        dna: *mut c_char,
        index: *mut I64,
        buf: *mut U64,
    ) -> bool;

    pub fn kmerHashWriteOneFile(kh: *mut KmerHash, of: *mut OneFile) -> bool;
    pub fn kmerHashReadOneFile(of: *mut OneFile) -> *mut KmerHash;

    /// Retrieve the i'th syncmer's DNA sequence (ASCII a/c/g/t lowercase).
    /// `i` is 1-based; negative i returns the reverse complement.
    /// `buf` must be at least `kh->len + 1` bytes (NUL terminator).
    /// Returns `buf` on success.
    pub fn kmerHashSeq(kh: *mut KmerHash, i: I64, buf: *mut c_char) -> *mut c_char;

    /// Re-exports of static-inline destroy helpers from seqhash.h.
    /// syng marks `seqhashDestroy` and `seqhashIteratorDestroy` `static`,
    /// preventing direct FFI calls. Our `syng_helpers.c` wraps them.
    pub fn syng_rs_seqhash_destroy(sh: *mut Seqhash);
    pub fn syng_rs_seqhash_iterator_destroy(si: *mut SeqhashIterator);
}

// ─── syng.h (SyngBWT API) ─────────────────────────────────────────────────────

extern "C" {
    pub fn syngBWTcreate(fixed_len: c_int, max: I64) -> *mut SyngBWT;
    pub fn syngBWTdestroy(sb: *mut SyngBWT);

    pub fn syngBWTwrite(of: *mut OneFile, sb: *mut SyngBWT);
    pub fn syngBWTread(of: *mut OneFile) -> *mut SyngBWT;

    pub fn syngBWTpathStartNew(sb: *mut SyngBWT, start_node: I32) -> *mut SyngBWTpath;
    pub fn syngBWTpathAdd(sbp: *mut SyngBWTpath, next_node: I32, offset: U32);
    pub fn syngBWTpathFinish(sbp: *mut SyngBWTpath);

    pub fn syngBWTpathStartOld(
        sb: *mut SyngBWT,
        start_node: I32,
        count: U32,
    ) -> *mut SyngBWTpath;
    pub fn syngBWTpathNext(
        sbp: *mut SyngBWTpath,
        next_node: *mut I32,
        next_pos: *mut U32,
    ) -> bool;

    pub fn syngBWTmatchStart(sb: *mut SyngBWT, start_node: I32, high: *mut U32) -> *mut SyngBWTpath;
    pub fn syngBWTmatchNext(
        sbp: *mut SyngBWTpath,
        next_node: I32,
        next_off: U32,
        low: *mut U32,
        high: *mut U32,
    ) -> bool;

    pub fn syngBWTpathDestroy(sbp: *mut SyngBWTpath);
    pub fn syngBWTstat(sb: *mut SyngBWT);

    pub fn syngBWTlocFind(
        sb: *mut SyngBWT,
        loc: I64,
        file: *mut I64,
        path: *mut I64,
        offset: *mut I64,
    ) -> bool;
}

// ─── ONElib.h ────────────────────────────────────────────────────────────────

extern "C" {
    pub fn oneSchemaCreateFromText(text: *const c_char) -> *mut OneSchema;
    pub fn oneSchemaDestroy(vs: *mut OneSchema);

    pub fn oneFileOpenRead(
        path: *const c_char,
        vs: *mut OneSchema,
        file_type: *const c_char,
        n_threads: c_int,
    ) -> *mut OneFile;
    pub fn oneFileClose(vf: *mut OneFile);

    /// Count records of `lineType` in the file.
    /// `count` is the number of records of that type; `max` is the max
    /// list-length seen across them; `total` is the sum of list lengths.
    /// `max` and `total` may be NULL if not needed. Returns true on success.
    pub fn oneStats(
        of: *mut OneFile,
        line_type: c_char,
        count: *mut I64,
        max: *mut I64,
        total: *mut I64,
    ) -> bool;

    /// Seek to the i'th record (1-based) of `lineType`. Returns true on success.
    pub fn oneGoto(of: *mut OneFile, line_type: c_char, i: I64) -> bool;

    /// Read the next line into the file's buffer; returns the line's type
    /// char (or 0 at end of file).
    pub fn oneReadLine(of: *mut OneFile) -> c_char;

    /// Field accessors (CPP macros in ONElib.h, re-exported as functions
    /// by `src/syng_helpers.c`).
    pub fn syng_rs_one_int(of: *mut OneFile, x: c_int) -> I64;
    /// Length of the current list field (e.g. for STRING / DNA / INT_LIST).
    pub fn syng_rs_one_len(of: *mut OneFile) -> I64;
    /// Pointer to the current STRING field's bytes (NUL-terminated for
    /// single strings; for string-lists use successive `oneNextString`
    /// pointer arithmetic — not yet bound).
    pub fn syng_rs_one_string(of: *mut OneFile) -> *const c_char;
    /// Pointer to the current INT_LIST field's I64 values; length via `syng_rs_one_len`.
    pub fn syng_rs_one_int_list(of: *mut OneFile) -> *const I64;
    /// Pointer to the current DNA field's chars.
    pub fn syng_rs_one_dna_char(of: *mut OneFile) -> *const c_char;
}

// syng.h defines `syngSchemaText` as a `static char *` global (internal
// linkage). Our `src/syng_helpers.c` re-exports it via a non-static accessor.
extern "C" {
    pub fn syng_rs_schema_text() -> *const c_char;
    /// Number of stored syncmers (kh->max). Wraps the kmerHashMax(kh) macro.
    pub fn syng_rs_kmer_hash_max(kh: *const KmerHash) -> I64;
    /// Syncmer length in bases (kh->len). Useful for sizing DNA buffers.
    pub fn syng_rs_kmer_hash_len(kh: *const KmerHash) -> c_int;
    /// Number of syncmer-graph nodes (arrayMax(sb->node)).
    pub fn syng_rs_gbwt_node_count(sb: *const SyngBWT) -> I64;
    /// Number of paths stored in the GBWT (arrayMax(sb->path)).
    pub fn syng_rs_gbwt_path_count(sb: *const SyngBWT) -> I64;
    /// Fixed syncmer length on the GBWT, or 0 if variable-length.
    pub fn syng_rs_gbwt_fixed_len(sb: *const SyngBWT) -> c_int;
    /// bp length of path `path_idx` (0-based; < gbwt_path_count).
    pub fn syng_rs_gbwt_path_length(sb: *const SyngBWT, path_idx: I64) -> U64;
    /// Source-file index of path `path_idx` (which input FASTA contributed it).
    pub fn syng_rs_gbwt_path_file(sb: *const SyngBWT, path_idx: I64) -> U32;
    /// Sequence index within the source file (1-based per syng's writer).
    pub fn syng_rs_gbwt_path_id(sb: *const SyngBWT, path_idx: I64) -> U32;

    /// Current GBWT rank of the cursor (sbp->jLast). Pass to
    /// `syngBWTpathStartOld(thisNode, jLast)` to re-enter at this point.
    pub fn syng_rs_path_jlast(sbp: *const SyngBWTpath) -> U32;
    /// Node the cursor is about to read next (sbp->thisNode).
    pub fn syng_rs_path_this_node(sbp: *const SyngBWTpath) -> I32;
    /// Node the cursor just read (sbp->lastNode).
    pub fn syng_rs_path_last_node(sbp: *const SyngBWTpath) -> I32;
    /// Offset of the last edge traversed (sbp->lastOff).
    pub fn syng_rs_path_last_off(sbp: *const SyngBWTpath) -> U32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Verifies the C library links and a Seqhash handle can be obtained.
    #[test]
    fn seqhash_create_links() {
        unsafe {
            let sh = seqhashCreate(8, 55, 7);
            assert!(!sh.is_null(), "seqhashCreate returned null");
            // seqhashDestroy is a static inline (header-only). We leak this
            // handle intentionally in this smoke test; production code uses
            // a safe wrapper or `libc::free` directly.
        }
    }

    /// Add a numeric-encoded 63-base k-mer; verify it round-trips via find.
    #[test]
    fn kmer_hash_add_find_round_trip() {
        unsafe {
            let kh = kmerHashCreate(64, 63);
            assert!(!kh.is_null(), "kmerHashCreate returned null");

            // syng encodes DNA as 0/1/2/3 (a/c/g/t) bytes. Build a numeric
            // buffer of length 63 (syncmer length for default k=8, w=55).
            let mut dna: Vec<i8> = vec![0; 64];
            for (i, b) in dna.iter_mut().take(63).enumerate() {
                *b = (i % 4) as i8;
            }

            let mut idx_add: I64 = 0;
            let added = kmerHashAdd(kh, dna.as_mut_ptr(), &mut idx_add);
            assert!(added, "kmerHashAdd should add a new k-mer");

            let mut idx_find: I64 = 0;
            let found = kmerHashFind(kh, dna.as_mut_ptr(), &mut idx_find);
            assert!(found, "kmerHashFind should find the k-mer just added");
            assert_eq!(idx_add, idx_find, "index must be stable across add/find");

            kmerHashDestroy(kh);
        }
    }

    /// Verifies the syng schema text accessor is linked and
    /// oneSchemaCreateFromText can parse it without crashing.
    #[test]
    fn schema_open_close() {
        unsafe {
            let text = syng_rs_schema_text();
            assert!(!text.is_null(), "syng_rs_schema_text returned null");
            let vs = oneSchemaCreateFromText(text);
            assert!(!vs.is_null(), "oneSchemaCreateFromText returned null");
            oneSchemaDestroy(vs);
        }
    }
}
