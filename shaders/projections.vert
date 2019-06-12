#version 450

#define pi 3.1415926535897932384626433832795

uniform float u_time;

// We have this here in order to avoid 5x5 matrices (technically, the would
// be the last column of this transformation matrix).
uniform vec4 u_four_from;

uniform mat4 u_four_model;
uniform mat4 u_four_view;
uniform mat4 u_four_projection;

uniform mat4 u_three_model;
uniform mat4 u_three_view;
uniform mat4 u_three_projection;

uniform bool u_perspective_4D;
uniform bool u_perspective_3D;

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 color;

out VS_OUT
{
    // A per-cell color
    vec3 color;

    // Model-space position, after projection from 4D -> 3D
    vec3 position;

    // Depth in the 4-th dimension
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

    // Project 4D -> 3D with a perspective projection.
    if (u_perspective_4D)
    {
        four = u_four_model * position;
        four = four - u_four_from;
        four = u_four_view * four;

        four = u_four_projection * four;
        four /= four.w;
    }
    // Project 4D -> 3D with a parallel (orthographic) projection.
    else
    {
        // Simply drop the last (w) coordinate.
        //four = u_four_model * position;
        //four = vec4(four.xyz, 1.0);

        // TODO: the code above doesn't always work, since the `u_four_model` matrix
        // TODO: is already applied to the slice vertices in the compute shader - we do a
        // TODO: standard orthographic projection for those vertices
        
        four = vec4(position.xyz, 1.0);
    }

    vec4 three;

    // Project 3D -> 2D with a perspective projection.
    if(perspective_3D)
    {
        three = u_three_projection * u_three_view * u_three_model * four;
    }
    // Project 3D -> 2D with a parallel (orthographic) projection.
    else
    {
        // TODO: is support for a 3D -> 2D orthographic projection useful / necessary?
    }

    gl_Position = three;
    gl_PointSize = 3.0;

    vec3 cell_centroid = color.rgb;
    vec3 centroid_color = normalize(cell_centroid) * 0.5 + 0.5;
    vec3 position_color = normalize(position.xyz) * 0.5 + 0.5;
    vec3 rgb = max(centroid_color, position_color);
    rgb = max(rgb, vec3(0.15));

    // Pass values to fragment shader.
    vs_out.color = rgb;
    vs_out.position = four.xyz;
}