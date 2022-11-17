fn main() {
    // This will consider the ffi part in lib.rs in order to
    // generate lib.rs.h and lib.rs.cc
    // minimal example: no C++ code to be called from Rust
    // compile(lib_name): Run the compiler, generating the file output, the param is the name of the library.
    // see: https://docs.rs/cc/1.0.49/cc/struct.Build.html#method.compile
    cxx_build::bridge("src/lib.rs")   // bridge the rust code into output library
        .flag_if_supported("-std=c++17")  // compile generated lib.rs.cc with c++17
        .compile("cpp_from_rust");  // run the compiler to generate the library
}
