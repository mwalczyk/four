#version 430
#extension GL_ARB_shading_language_420pack : enable

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

uniform vec4 u_cell_centroid;

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

out VS_OUT 
{
    // a per-cell color
    vec3 color;

    // model-space position, after projection from 4D -> 3D
    vec3 position;
} vs_out;

float sigmoid(float x) 
{
    return 1.0 / (1.0 + exp(-x));
}

vec3 hsb2rgb(in vec3 c)
{
    vec3 rgb = clamp(abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),
                             6.0)-3.0)-1.0,
                     0.0,
                     1.0 );
    rgb = rgb * rgb * (3.0 - 2.0 * rgb);
    return c.z * mix(vec3(1.0), rgb, c.y);
}

void main()
{
    // drop the last coordinate (w) and prepare for 3D -> 2D projection
    vec4 projected = vec4(position.xyz, 1.0);

    // create a color based on the centroid of this cell in 4D
    vec3 cell = color.rgb; //TODO u_cell_centroid.xyz;
    vec3 rgb = normalize(cell) * 0.5 + 0.5;
    rgb = max(vec3(0.15), rgb);

    // project 3D -> 2D
    gl_Position = u_three_projection * u_three_view * u_three_rotation * projected;
    gl_PointSize = 6.0;

    // pass values to fragment shader
    vs_out.color = rgb;
    vs_out.position = projected.xyz;
}