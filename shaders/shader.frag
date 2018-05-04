#version 430
#extension GL_ARB_shading_language_420pack : enable

uniform vec4 u_draw_color;

in VS_OUT 
{
    vec3 color;
    vec3 position;
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
    o_color = vec4(fs_in.color.rgb, 1.0);
}