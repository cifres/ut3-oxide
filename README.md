# ut3-oxide

## Useful Commands
`cargo rustc --release -- emit {mir | llvm-ir | asm}`
`rustc main.rs -O --emit {mir | llvm-ir | asm}`
`rustc main.rs -C opt-level=3 --emit {mir | llvm-ir | asm}`
`cargo rustc --release -- -C opt-level=3 -C -target-cpu=native --emit {mir | llvm-ir | asm}`
```bash
rustc --print target-cpus --target=x86_64-pc-windows-msvc
```
```bash
cargo rustc --bin ut3_oxide --release -- -C opt-level=3 -C target-cpu=native -Cllvm-args="--x86-asm-syntax=intel" --emit=asm
```
```bash
RUSTFLAGS="-C target-cpu=znver4 -C opt-level=3" cargo +nightly build --profile release-fast --target=x86_64-pc-windows-msvc 
```
