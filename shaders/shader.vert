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

uniform vec4 u_cell_centroid;

layout(location = 0) in vec4 position;

out VS_OUT 
{
    float depth;
    vec3 color;
} vs_out;

float sigmoid(float x) 
{
    return 1.0 / (1.0 + exp(-x));
}

vec3 hsb2rgb( in vec3 c ){
    vec3 rgb = clamp(abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),
                             6.0)-3.0)-1.0,
                     0.0,
                     1.0 );
    rgb = rgb*rgb*(3.0-2.0*rgb);
    return c.z * mix( vec3(1.0), rgb, c.y);
}

void main()
{
    const float scale = 0.75;

    // drop the last coordinate (w) and prepare for 3D -> 2D projection
    vec4 p = vec4(position.xyz * scale, 1.0);
    float depth_cue = position.z * 0.5 + 0.5;

    // create a color based on the centroid of this cell in 4D
    vec3 centr = u_cell_centroid.xyz * scale;
    float h = atan(centr.z, centr.x) / (2.0 * pi);
    float s = 0.75;
    float b = max(0.15, centr.y * 0.5 + 0.5);
    vec3 rgb = hsb2rgb(vec3(h, s, b));

    // project 3D -> 2D
    gl_Position = u_three_projection * u_three_view * u_three_rotation * p;
    gl_PointSize = 6.0;

    // pass 4D depth to fragment shader
    vs_out.depth = depth_cue;
    vs_out.color = rgb;
}