#version 450

#include "tonemapping.glsl"

layout(location = 0) in vec2 f_uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

#define TONEMAP_NONE 0
#define TONEMAP_SIMPLE_ACES 1
#define TONEMAP_SIMPLE_REINHARD 2
#define TONEMAP_LUMA_REINHARD 3
#define TONEMAP_LUMA_WHITEPRESERVING_REINHARD 4
#define TONEMAP_ROM_BIN_DA_HOUSE 5
#define TONEMAP_FILMIC 6
#define TONEMAP_UNCHARTED_2 7

layout(push_constant) uniform PostProcessingData {
  float scale;
  float gamma;
  float exposure;
  int tonemap;
  bool transparent;
} ppd;

void main() {
    vec4 sampled = texture(tex, f_uv) * ppd.scale;
    vec3 color = sampled.xyz * ppd.exposure;

    float gamma = ppd.gamma;

    switch (ppd.tonemap) {
      case TONEMAP_SIMPLE_ACES:
        color = tonemap_simple_ACES(color);
        break;
      case TONEMAP_SIMPLE_REINHARD:
        color = tonemap_simple_reinhard(color);
        break;
      case TONEMAP_LUMA_REINHARD:
        color = tonemap_luma_reinhard(color);
        break;
      case TONEMAP_LUMA_WHITEPRESERVING_REINHARD:
        color = tonemap_luma_whitepreserving_reinhard(color);
        break;
      case TONEMAP_ROM_BIN_DA_HOUSE:
        color = tonemap_rombindahouse(color);
        break;
      case TONEMAP_FILMIC:
        color = tonemap_filmic(color);
        // Filmic doesn't have gamma
        gamma = 1.0;
        break;
      case TONEMAP_UNCHARTED_2:
        color = tonemap_uncharted2(color);
        break;
      default:
        break;
    }

    color = pow(color, vec3(1.0 / gamma));

    if (ppd.transparent) {
        f_color = vec4(color, sampled.w);
    } else {
        f_color = vec4(color, 1.0);
    }
}
