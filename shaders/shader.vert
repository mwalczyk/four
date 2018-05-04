#version 430
#extension GL_ARB_shading_language_420pack : enable

const float pi = 3.1415926535897932384626433832795;

uniform float u_time;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

out VS_OUT 
{
    vec3 position;
    vec3 color;
} vs_out;

float sigmoid(float x) 
{
    return 1.0 / (1.0 + exp(-x));
}

void main()
{
    // Create a color based on the centroid of this cell in 4D.
    vec3 cell_centroid = color.rgb;
    vec3 rgb = normalize(cell_centroid) * 0.5 + 0.5;
    rgb = max(vec3(0.15), rgb);

    // Project 3D -> 2D.
    gl_Position = u_projection * u_view * u_model * position;
    gl_PointSize = 6.0;

    // Pass values to fragment shader.
    vs_out.position = position.xyz;
    vs_out.color = rgb;
}