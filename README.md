# ut3-oxide

## Useful Commands
`cargo rustc --release -- emit {mir | llvm-ir | asm}`
`rustc main.rs -O --emit {mir | llvm-ir | asm}`
`rustc main.rs -C opt-level=3 --emit {mir | llvm-ir | asm}`
`cargo rustc --release -- -C opt-level=3 -C -target-cpu=native --emit {mir | llvm-ir | asm}`
```bash
cargo rustc --bin ut3_oxide --release -- -C opt-level=3 -C target-cpu=native -C codegen-units=1 -Cllvm-args=
"--x86-asm-syntax=intel" --emit=asm
```
