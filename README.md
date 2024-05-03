# SPar-Rust (version 1)

This repository contains the source code for the first version of SPar-Rust. A Domain-Specific Language for expressing stream parallelism (linear pipelines) in Rust programs targeting multi-cores.
The main goal is to reduce the parallel programming effort. It was inspired by [SPar](https://github.com/GMAP/SPar), which is a C++ annotation-based Domain-Specific Language for expressing stream parallelism.


## How to cite
- [DOI](https://dl.acm.org/doi/10.1145/3624309.3624320) Leonardo G. Faé, Renato B. Hoffman, and Dalvan Griebler. 2023. **Source-to-Source Code Transformation on Rust for High-Level Stream Parallelism**. *In Proceedings of the XXVII Brazilian Symposium on Programming Languages (SBLP '23)*. Association for Computing Machinery, New York, NY, USA, 41–49. 

### Using SPar-Rust v1

First, include it as a dependency in `Cargo.toml`:

```toml
spar-rust = {git = "https://github.com/GMAP/SPar-Rust.git", tag = "v0.1.0" }
```

Then, you can call it like so:

```rust
to_stream!(INPUT(input: Vec<Item>), {
    for item in input.into_iter() {
        STAGE(
            INPUT(item: Item),
            OUTPUT(item: Item),
            REPLICATE = 2, // run in 2 threads
            {
            // code that transforms ITEM
            }
        );
        // you can put in as many stages as you want, as long as the INPUTs and OUTPUTs all line up
        STAGE(
            INPUT(item: Item)
            ...
        );
        // The final STAGE *cannot have an OUTPUT*
        STAGE(
            INPUT(item: Item),
            ORDERED, //optionally, set the ordered flag if you want the result to have the same order as the original input
            {
                 // code for final pipeline stage
            }
        );
    }
}
```

As long you can get the code to compile, it should behave correctly. Nevertheless, do not blindly trust this.
This code was made primarily as a proof of concept, and it is not meant to be used in production. Furthermore,
version 1 of this library (that you are currently seeing) has been superseded by version 2, which is better in
nearly all aspects.
