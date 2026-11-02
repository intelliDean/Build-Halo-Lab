# Build Halo Lab vendored changes

Upstream: https://github.com/tachyon-zcash/ragu
Commit: 4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5

Retained upstream MIT/Apache-2.0 licensing and COPYRIGHT files.

Lab changes:
- Add crates/ragu_pcd/src/compress/codec.rs: canonical compressed-proof transport.
- Add `mod codec;` in crates/ragu_pcd/src/compress/mod.rs.
- Materialize crates/*/katex-header.html symlinks using their original target
  contents, because archive extraction of those links is unsupported on this
  Windows environment. This affects documentation only.

No recursion, proving, transcript, commitment, accumulation, or verification
algorithm is modified. The transport adapter is original educational lab work;
it has not been accepted or reviewed by upstream. See ../../docs/ragu-api.md
for the byte format and exact API differences from the current book.
