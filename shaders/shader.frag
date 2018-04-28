#version 430

#define pi 3.1415926535897932384626433832795

uniform vec4 u_draw_color;

in VS_OUT 
{
    float depth;
} fs_in;

layout(location = 0) out vec4 o_color;

void round_point_sprite()
{
    if(length(gl_PointCoord - vec2(0.5)) > 0.5)
    {
        discard;
    }
}

void main()
{
    o_color = u_draw_color;
}