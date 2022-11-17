fn main() {
    // This will consider the ffi part in lib.rs in order to
    // generate lib.rs.h and lib.rs.cc
    // minimal example: no C++ code to be called from Rust
    cxx_build::bridge("src/lib.rs")
        .compile("cpp_from_rust");
}
