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
    vec4 color;

    // Model-space position, after projection from 4D -> 3D
    vec3 position;

    // Depth in the 4-th dimension
    float depth_cue;
} vs_out;

// https://github.com/hughsk/glsl-hsv2rgb/blob/master/index.glsl
vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
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

        // TODO: the code above doesn't always work, since the `u_four_model` matrix (rotations)
        // TODO: is already applied to the tetrahedra vertices in the compute shader prior to
        // TODO: generating the 3D slice - we do a standard orthographic projection for this
        // TODO: draw mode, instead
        
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

    // Original shading mode...
    vec3 rgb = max(centroid_color, position_color);

    // New shading mode (shade wireframes and slices differently)...
    rgb = u_perspective_4D ? position_color : centroid_color;
    rgb = max(rgb, vec3(0.15));
    float alpha = u_perspective_4D ? 0.5 : 1.0;

    // Pass values to fragment shader.
    vs_out.color = vec4(rgb, alpha);
    vs_out.position = four.xyz;
}