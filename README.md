# Voidray

High performance, interactive, physically based Rust path tracer.

<img width="1490" alt="Screen Shot 2022-09-12 at 1 23 30 AM" src="https://user-images.githubusercontent.com/13054020/189579542-d5aaa2cc-8555-42d0-8331-c90b993f8ec5.png">

## Features

- 100% Safe Rust
- Non-blocking UI with progressive rendering
- Vulkano backend
- Supports various tonemapping functions such as ACES and Filmic
- Supports HDRI environment maps
- Uses BVH trees to accelerate ray-mesh intersections
- Optimally uses all CPU cores in multithreaded rendering
