# Voidray

High performance, interactive, physically based Rust path tracer.

<img width="1495" alt="Screen Shot 2022-09-12 at 12 16 30 PM" src="https://user-images.githubusercontent.com/13054020/190555683-fbedb0a7-074b-4e41-8abe-840db0fd5dd9.png">
<img width="1191" alt="Screen Shot 2022-09-07 at 2 44 02 AM" src="https://user-images.githubusercontent.com/13054020/190555622-db5b125c-0ec0-4493-912e-03ca986a8314.png">

## Running

To run,

```sh
cargo run --release --bin=voidray_app
```
Make sure Vulkan is installed with dev dependencies and all the necessary dependencies for compiling https://github.com/vulkano-rs/vulkano are installed. (e.g. cmake, python, etc...)

## Features

- 100% Safe Rust
- Non-blocking UI with progressive rendering
- Vulkano backend
- Supports various tonemapping functions such as ACES and Filmic
- Supports HDRI environment maps
- Uses BVH trees to accelerate ray-mesh intersections
- Optimally uses all CPU cores in multithreaded rendering
