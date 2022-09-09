#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(tex, f_uv);
}
