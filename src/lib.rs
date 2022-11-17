#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn rust_from_cpp() -> ();
    }
}

pub fn rust_from_cpp() -> () {
    println!("called rust_from_cpp()");
}
