#version 450

#define pi 3.1415926535897932384626433832795

uniform float u_time;

// we have this here in order to avoid 5x5 matrices (technically, the would
// be the last column of this transformation matrix)
uniform vec4 u_four_from;

uniform mat4 u_four_model;
uniform mat4 u_four_view;
uniform mat4 u_four_projection;

uniform mat4 u_three_model;
uniform mat4 u_three_view;
uniform mat4 u_three_projection;

uniform uint u_perspective_4D;
uniform uint u_perspective_3D;

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
    vs_out.depth_cue = position.w;

    // project 4D -> 3D with a perspective projection
    if (u_perspective_4D == 1)
    {
        four = u_four_model * position;
        four = four - u_four_from;
        four = u_four_view * four;

        four = u_four_projection * four;
        four /= four.w;
    }
    // project 4D -> 3D with a parallel (orthographic) projection
    else
    {
        // simply drop the last (w) coordinate
        //four = u_four_model * position;
        //four = vec4(four.xyz, 1.0);
        // TODO: the code above doesn't always work, since the `u_four_model` matrix
        // is already applied to the slice vertices in the compute shader...we do a
        // standard orthographic projection for those vertices
        
        four = vec4(position.xyz, 1.0);
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

    vec3 cell_centroid = color.rgb;
    vec3 cen = normalize(cell_centroid) * 0.5 + 0.5;
    vec3 rgb = normalize(position.xyz) * 0.5 + 0.5;
    rgb = max(cen, rgb);

    // pass values to fragment shader
    vs_out.color = rgb;// vec3(0.0, 1.0, 0.0);
    vs_out.position = four.xyz;
    vs_out.depth_cue = vs_out.depth_cue * 0.5 + 0.5;
}