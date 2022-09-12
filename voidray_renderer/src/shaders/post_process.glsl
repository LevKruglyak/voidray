#version 450

#include "tonemapping.glsl"

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba32f) uniform readonly image2D src;
layout(set = 0, binding = 1, rgba32f) uniform writeonly image2D dst;

layout(push_constant) uniform PostProcessingData {
  float scale;
  float gamma;
  float exposure;
  int tonemap;
} ppd;

#define TONEMAP_NONE 0
#define TONEMAP_ACES 1
#define TONEMAP_REINHARD 2
#define TONEMAP_FILMIC 3
#define TONEMAP_UNCHARTED_2 4

void main() {
    ivec2 pos = ivec2(gl_GlobalInvocationID.xy);
    vec4 sampled = imageLoad(src, pos) * ppd.scale;

    vec3 color = sampled.xyz * pow(2, ppd.exposure);

    switch (ppd.tonemap) {
      case TONEMAP_ACES:
        color = tonemap_ACES(color);
        break;
      case TONEMAP_REINHARD:
        color = tonemap_reinhard(color);
        break;
      case TONEMAP_FILMIC:
        color = tonemap_filmic(color);
        break;
      case TONEMAP_UNCHARTED_2:
        color = tonemap_uncharted2(color);
        break;
      default:
        break;
    }
    
    color = pow(color, vec3(1.0 / ppd.gamma));

    imageStore(dst, pos, vec4(color, 1.0));
}
