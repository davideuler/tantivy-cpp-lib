# rust-lib-via-cpp

The project is a minimal demo for calling Rust library in C++.


## Build rust lib
```
cargo build
```

## Build c++ demo application:
```
g++ -std=c++17 -o cpp_program src/main.cpp \
      -I .. -I target/cxxbridge \
      -L target/debug -l arustlib \
      -pthread -l dl
```


Reference: 
* https://stackoverflow.com/questions/71097948/failing-to-use-cxx-to-link-rust-written-library-in-c-project

* https://cxx.rs/
