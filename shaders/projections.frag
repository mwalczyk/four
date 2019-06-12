#version 450

in VS_OUT 
{
    vec3 color;
    vec3 position;
    float depth_cue;
} fs_in;

layout(location = 0) out vec4 o_color;

void main()
{
    float modified_depth = fs_in.depth_cue;

    o_color = vec4(fs_in.color, 1.0);// vec4(vec3(modified_depth, 0.5, 1.0), 0.25);
}