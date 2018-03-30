#version 430

#define pi 3.1415926535897932384626433832795

in VS_OUT 
{
    float depth;
} fs_in;

layout(location = 0) out vec4 o_color;

vec3 palette(in float t, in vec3 a, in vec3 b, in vec3 c, in vec3 d)
{
    return a + b * cos(2.0 * pi * (c * t + d));
}

void main() {
    vec3 c = palette(fs_in.depth * 4.0 - 0.85,
                     vec3(0.50, 0.70, 0.50),
                     vec3(0.50, 0.50, 0.50),
                     vec3(1.00, 1.00, 1.00),
                     vec3(0.00, 0.10, 0.20));
//
//    vec3 c = palette(fs_in.depth * 4.0,
//                     vec3(0.70, 0.60, 0.50),
//                     vec3(0.20, 0.40, 0.20),
//                     vec3(1.00, 1.00, 1.00),
//                     vec3(0.40, 0.25, 0.25));

    vec3 color = vec3(fs_in.depth);
    o_color = vec4(c, 1.0);
}