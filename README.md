# Voidray

High performance, interactive, physically based Rust path tracer.

<img width="1495" alt="Screen Shot 2022-09-12 at 2 48 38 AM" src="https://user-images.githubusercontent.com/13054020/189590689-e3983a4a-fb25-451a-bcd8-ef69614706da.png">
<img width="1191" alt="Screen Shot 2022-09-07 at 2 44 02 AM" src="https://user-images.githubusercontent.com/13054020/190555622-db5b125c-0ec0-4493-912e-03ca986a8314.png">


## Features

- 100% Safe Rust
- Non-blocking UI with progressive rendering
- Vulkano backend
- Supports various tonemapping functions such as ACES and Filmic
- Supports HDRI environment maps
- Uses BVH trees to accelerate ray-mesh intersections
- Optimally uses all CPU cores in multithreaded rendering
