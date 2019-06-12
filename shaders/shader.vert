#version 450

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

// Cosine palette generator from IQ: `http://www.iquilezles.org/www/articles/palettes/palettes.htm`
vec3 palette( in float t, in vec3 a, in vec3 b, in vec3 c, in vec3 d )
{
    return a + b * cos(2.0 * pi * (c * t + d));
}

void main()
{
    // Create a color based on the centroid of this cell in 4D.
    vec3 cell_centroid = color.rgb;
    //vec3 rgb = normalize(cell_centroid) * 0.5 + 0.5;
    //rgb = max(vec3(0.15), rgb);

    // Project 3D -> 2D.
    vec4 p = vec4(position.xyz, 1.0);
    gl_Position = u_projection * u_view * u_model * p;
    gl_PointSize = 6.0;

    vec3 cen = normalize(cell_centroid) * 0.5 + 0.5;
    vec3 rgb = normalize(position.xyz) * 0.5 + 0.5;
    rgb = max(cen, rgb);

    // Pass values to fragment shader.
    vs_out.position = position.xyz;
    vs_out.color = rgb;
}