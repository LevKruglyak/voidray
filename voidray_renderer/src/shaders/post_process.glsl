#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} data;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    data.data[idx] *= 12;
}
