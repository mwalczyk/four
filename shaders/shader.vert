#version 430

#define pi 3.1415926535897932384626433832795

uniform float u_time;

// we have this here in order to avoid 5x5 matrices
uniform vec4 u_four_from;

uniform mat4 u_four_rotation;
uniform mat4 u_four_view;
uniform mat4 u_four_projection;

uniform mat4 u_three_rotation;
uniform mat4 u_three_view;
uniform mat4 u_three_projection;

layout(location = 0) in vec4 position;

out VS_OUT 
{
    float depth;
} vs_out;

float sigmoid(float x) 
{
    return 1.0 / (1.0 + exp(-x));
}

void main()
{
    float depth_cue = position.w;

    // drop the last coordinate (w) and prepare for 3D -> 2D projection
    vec4 p = vec4(position.xyz * 0.75, 1.0);

    // project 3D -> 2D
    gl_Position = u_three_projection * u_three_view * u_three_rotation * p;
    gl_PointSize = 6.0;

    // pass 4D depth to fragment shader
    vs_out.depth = 1.0;
}