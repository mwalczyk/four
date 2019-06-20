#version 450

in VS_OUT 
{
    vec4 color;
    vec3 position;
    float depth_cue;
} fs_in;

layout(location = 0) out vec4 o_color;

void main()
{
    o_color = fs_in.color;
}