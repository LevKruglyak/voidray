#version 450

#include "utils.glsl"

layout(location = 0) in vec2 f_uv;
layout(location = 0) out vec4 f_color;

layout(push_constant) uniform CheckerboardData {
    float width;
    float height;
} cd;

void main()
{
    float size = 20.0;
    vec2 pos = floor(f_uv * vec2(cd.width, cd.height) / size);
    float mask = mod(pos.x + mod(pos.y, 2.0), 2.0);
    f_color = vec4(vec3(map(mask, 0.0, 1.0, 0.05, 0.10)), 1.0);
}
