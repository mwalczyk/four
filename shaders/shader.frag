#version 430

#define pi 3.1415926535897932384626433832795

uniform vec4 u_draw_color;
uniform vec4 u_cell_centroid;

in VS_OUT 
{
    float depth;
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
    vec3 n = normalize(u_cell_centroid.xyz * 0.75);

    const vec3 l = vec3(5.0, 3.0, -3.0);
    vec3 to_l = normalize(l - fs_in.position);
    float diffuse = max(0.0, dot(n, to_l));

    o_color = vec4(fs_in.color.rgb, 1.0);
}