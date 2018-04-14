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

#define ORTHOGRAPHIC

void main() {
    // project 4D -> 3D

#ifdef ORTHOGRAPHIC
    // reference: https://en.wikipedia.org/wiki/User:Tetracube/Coordinates_of_uniform_polytopes#Mapping_coordinates_back_to_n-space
    const float n = 4.0;
    mat4 rot = mat4(
        vec4(sqrt(1.0 / n), -sqrt((n - 1.0) / n),          0.0,                                  0.0),
        vec4(sqrt(1.0 / n),  sqrt(1.0 / (n * (n - 1.0))), -sqrt((n - 2.0) / (n - 1.0)),          0.0),
        vec4(sqrt(1.0 / n),  sqrt(1.0 / (n * (n - 1.0))),  sqrt(1.0 / ((n - 1.0) * (n - 2.0))), -sqrt((n - 3.0) / (n - 2.0))),
        vec4(sqrt(1.0 / n),  sqrt(1.0 / (n * (n - 1.0))),  sqrt(1.0 / ((n - 1.0) * (n - 2.0))),  sqrt(1.0 / ((n - 2.0) * (n - 3.0))))
    );

    vec4 p = rot * position;

    // drop the first coordinate (x)
    p.xyz = p.yzw;


//    p = u_four_rotation * position;
//    p = p - u_four_from;
//    p = u_four_view * p;
    float depth_cue = p.w;
    p.w = 1.0;

#else
    vec4 p = u_four_rotation * position;
    p = p - u_four_from;
    p = u_four_view * p;
    float depth_cue = p.w;

    p = u_four_projection * p;
    p /= p.w;
#endif

    // project 3D -> 2D
    gl_Position = u_three_projection * u_three_view * u_three_rotation * p;
    gl_PointSize = 8.0; //depth_cue * 4.0;

    // pass 4D depth to fragment shader
    vs_out.depth = sigmoid(depth_cue);
}