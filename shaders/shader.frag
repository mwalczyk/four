#version 430

#define pi 3.1415926535897932384626433832795

uniform vec4 u_draw_color;

in VS_OUT 
{
    float depth;
    vec3 color;
} fs_in;

layout(location = 0) out vec4 o_color;

void round_point_sprite()
{
    if(length(gl_PointCoord - vec2(0.5)) > 0.5)
    {
        discard;
    }
}

vec4 desaturate(vec3 color, float amount)
{
	vec3 gray_transfer = vec3(0.3, 0.59, 0.11);
	vec3 gray = vec3(dot(gray_transfer, color));
	return vec4(mix(color, gray, amount), 1.0);
}

void main()
{
    o_color = desaturate(fs_in.color.rgb, 0.0);
    o_color.a = u_draw_color.a;

   // o_color = u_draw_color;
}