# Build and profile
```bash
cargo build --profile=release-with-debug # build
valgrind --tool=callgrind target/release-with-debug/rusty-solver # profile
kcachegrind callgrind.out.<PID> # view profile
hyperfine target/release-with-debug/rusty-solver # benchmark
```