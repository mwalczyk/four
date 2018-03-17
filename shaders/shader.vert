#version 430

#define pi 3.1415926535897932384626433832795

uniform mat4 u_four_view;
uniform mat4 u_four_projection;
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
    // project 4D -> 3D
    if (false)
    {
        // vec4 pos = position;
        // float t = 1.0 / tan(pi * 0.25 * 0.5);
        // vec4 v = pos - u_four_from;
        // float s = t / dot(v, u_four_view[3]);
        // pos.x = s * dot(v, u_four_view[0]);
        // pos.y = s * dot(v, u_four_view[1]);
        // pos.z = s * dot(v, u_four_view[2]);
        // pos.w = 1.0;
    }

    // project 3D -> 2D
    gl_Position = u_three_projection * u_three_view * position;
    gl_PointSize = depth_cue * 4.0;

    // pass 4D depth to fragment shader
    vs_out.depth = sigmoid(depth_cue);
}