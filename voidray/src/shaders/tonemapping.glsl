// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 tonemap_simple_ACES(vec3 color)
{
    // float a = 2.51;
    // float b = 0.03;
    // float c = 2.43;
    // float d = 0.59;
    // float e = 0.14;
    // color = clamp((color*(a*color+b))/(color*(c*color+d)+e), 0.0, 1.0);
    // return color;

    color = mat3(0.59719, 0.076, 0.0284, 0.35458, 0.90834, 0.13383, 0.04823, 0.01566, 0.83777) * color;
    color = (color * (color + 0.0245786) - 0.000090537) / (color * (0.983729 * color + 0.432951) + 0.238081);
    return mat3(1.60475, -0.10208, -0.00327, -0.53108, 1.10813, -0.07276, -0.07367, -0.00605, 1.07602) * color;
}

vec3 tonemap_simple_reinhard(vec3 color)
{
	float exposure = 1.5;
	color *= exposure/(1. + color / exposure);
	return color;
}

vec3 tonemap_luma_reinhard(vec3 color)
{
	float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
	float toneMappedLuma = luma / (1. + luma);
	color *= toneMappedLuma / luma;
	return color;
}

vec3 tonemap_luma_whitepreserving_reinhard(vec3 color)
{
	float white = 2.;
	float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
	float toneMappedLuma = luma * (1. + luma / (white*white)) / (1. + luma);
	color *= toneMappedLuma / luma;
	return color;
}

vec3 tonemap_rombindahouse(vec3 color)
{
    color = exp( -1.0 / ( 2.72*color + 0.15 ) );
	return color;
}

vec3 tonemap_filmic(vec3 color)
{
	color = max(vec3(0.), color - vec3(0.004));
	color = (color * (6.2 * color + .5)) / (color * (6.2 * color + 1.7) + 0.06);
	return color;
}

vec3 tonemap_uncharted2(vec3 color)
{
	float A = 0.15;
	float B = 0.50;
	float C = 0.10;
	float D = 0.20;
	float E = 0.02;
	float F = 0.30;
	float W = 11.2;
	float exposure = 2.;
	color *= exposure;
	color = ((color * (A * color + C * B) + D * E) / (color * (A * color + B) + D * F)) - E / F;
	float white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
	color /= white;
	return color;
}
