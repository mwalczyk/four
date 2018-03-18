#version 430

#define pi 3.1415926535897932384626433832795

uniform float u_time;

uniform mat4 u_four_view;
uniform vec4 u_four_from;
uniform mat4 u_four_projection;
uniform mat4 u_four_rotation;

uniform mat4 u_three_rotation;
uniform mat4 u_three_view;
uniform mat4 u_three_projection;

layout(location = 0) in vec4 position;
layout(location = 1) in float depth_cue;

out VS_OUT 
{
    float depth;
} vs_out;

float linear_depth(float z, float n, float f)
{
    float linear = 2.0 * z - 1.0;
    linear = 2.0 * n * f / (f + n - linear * (f - n));
    
    return linear;
}

float sigmoid(float x) 
{
    return 1.0 / (1.0 + exp(-x));
}

void main() {
    vec4 p = position;

    // project 4D -> 3D
    float t = 1.0 / tan(pi * 0.25 * 0.5);
    vec4 temp = (u_four_rotation * p) - u_four_from;
    float s = t / dot(temp, u_four_view[3]);

    p.x = s * dot(temp, u_four_view[0]);
    p.y = s * dot(temp, u_four_view[1]);
    p.z = s * dot(temp, u_four_view[2]);
    p.w = 1.0;

    // project 3D -> 2D
    gl_Position = u_three_projection * u_three_view * u_three_rotation * p;
    gl_PointSize = s * 4.0;

    // pass 4D depth to fragment shader
    vs_out.depth = sigmoid(s);
}