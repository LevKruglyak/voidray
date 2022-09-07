# Voidray

High performance, interactive, physically based Rust path tracer.

<img width="1191" alt="Screen Shot 2022-09-07 at 8 19 07 AM" src="https://user-images.githubusercontent.com/13054020/188923883-cc6a4492-3778-4b34-8857-fa25f559bd03.png">

## Features

- 100% Safe Rust
- Non-blocking UI with progressive rendering
- Vulkano backend
- Supports various tonemapping functions such as ACES and Filmic
- Supports HDRI environment maps
- Uses BVH trees to accelerate ray-mesh intersections
- Optimally uses all CPU cores in multithreaded rendering
