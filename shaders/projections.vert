#version 450

#define pi 3.1415926535897932384626433832795

uniform float u_time;

// we have this here in order to avoid 5x5 matrices (technically, the would
// be the last column of this transformation matrix)
uniform vec4 u_four_from;

uniform mat4 u_four_rotation;
uniform mat4 u_four_view;
uniform mat4 u_four_projection;

uniform mat4 u_three_model;
uniform mat4 u_three_view;
uniform mat4 u_three_projection;

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

out VS_OUT
{
    // a per-cell color
    vec3 color;

    // model-space position, after projection from 4D -> 3D
    vec3 position;

    // the depth in the 4-th dimension
    float depth_cue;
} vs_out;


float sigmoid(float x)
{
    return 1.0 / (1.0 + exp(-x));
}

void main()
{
    bool perspective_4D = false;
    bool perspective_3D = true;

    vec4 four;

    // project 4D -> 3D with a perspective projection
    if (perspective_4D)
    {
        four = u_four_rotation * position;
        four = four - u_four_from;
        four = u_four_view * four;
        vs_out.depth_cue = four.w;

        four = u_four_projection * four;
        four /= four.w;
    }
    // project 4D -> 3D with a parallel (orthographic) projection
    else
    {
        // simply drop the last (w) coordinate
        four = u_four_rotation * position;
        four = vec4(four.xyz, 1.0);
        vs_out.depth_cue = four.w;
    }

    vec4 three;

    // project 3D -> 2D with a perspective projection
    if(perspective_3D)
    {
        three = u_three_projection * u_three_view * u_three_model * four;
    }
    // project 3D -> 2D with a parallel (orthographic) projection
    else
    {
        // TODO
    }

    gl_Position = three;
    gl_PointSize = 3.0;

    // pass values to fragment shader
    vs_out.color = vec3(0.0, 1.0, 0.0);
    vs_out.position = four.xyz;
    vs_out.depth_cue = sigmoid(vs_out.depth_cue);
}