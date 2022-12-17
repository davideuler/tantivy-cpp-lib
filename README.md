# Tantivy static lib bridge for calling in c++

The project is tantivy static library for calling in c++


## Build rust lib
```
cargo build
```

## Build c++ demo application:
```
g++ -std=c++17 -o index_program src/main.cpp \
      -I .. -I target/cxxbridge \
      -L target/debug -l tantivy-cpp-lib \
      -pthread -l dl
```

## run the demo c++ application

```
./index_program
```

Reference: 
* https://stackoverflow.com/questions/71097948/failing-to-use-cxx-to-link-rust-written-library-in-c-project

* https://cxx.rs/
