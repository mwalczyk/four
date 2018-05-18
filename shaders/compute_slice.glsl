#version 430
#extension GL_ARB_shading_language_420pack : enable

vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec4 taylorInvSqrt(vec4 r){return 1.79284291400159 - 0.85373472095314 * r;}
vec4 fade(vec4 t) {return t*t*t*(t*(t*6.0-15.0)+10.0);}

float cnoise(vec4 P){
  vec4 Pi0 = floor(P); // Integer part for indexing
  vec4 Pi1 = Pi0 + 1.0; // Integer part + 1
  Pi0 = mod(Pi0, 289.0);
  Pi1 = mod(Pi1, 289.0);
  vec4 Pf0 = fract(P); // Fractional part for interpolation
  vec4 Pf1 = Pf0 - 1.0; // Fractional part - 1.0
  vec4 ix = vec4(Pi0.x, Pi1.x, Pi0.x, Pi1.x);
  vec4 iy = vec4(Pi0.yy, Pi1.yy);
  vec4 iz0 = vec4(Pi0.zzzz);
  vec4 iz1 = vec4(Pi1.zzzz);
  vec4 iw0 = vec4(Pi0.wwww);
  vec4 iw1 = vec4(Pi1.wwww);

  vec4 ixy = permute(permute(ix) + iy);
  vec4 ixy0 = permute(ixy + iz0);
  vec4 ixy1 = permute(ixy + iz1);
  vec4 ixy00 = permute(ixy0 + iw0);
  vec4 ixy01 = permute(ixy0 + iw1);
  vec4 ixy10 = permute(ixy1 + iw0);
  vec4 ixy11 = permute(ixy1 + iw1);

  vec4 gx00 = ixy00 / 7.0;
  vec4 gy00 = floor(gx00) / 7.0;
  vec4 gz00 = floor(gy00) / 6.0;
  gx00 = fract(gx00) - 0.5;
  gy00 = fract(gy00) - 0.5;
  gz00 = fract(gz00) - 0.5;
  vec4 gw00 = vec4(0.75) - abs(gx00) - abs(gy00) - abs(gz00);
  vec4 sw00 = step(gw00, vec4(0.0));
  gx00 -= sw00 * (step(0.0, gx00) - 0.5);
  gy00 -= sw00 * (step(0.0, gy00) - 0.5);

  vec4 gx01 = ixy01 / 7.0;
  vec4 gy01 = floor(gx01) / 7.0;
  vec4 gz01 = floor(gy01) / 6.0;
  gx01 = fract(gx01) - 0.5;
  gy01 = fract(gy01) - 0.5;
  gz01 = fract(gz01) - 0.5;
  vec4 gw01 = vec4(0.75) - abs(gx01) - abs(gy01) - abs(gz01);
  vec4 sw01 = step(gw01, vec4(0.0));
  gx01 -= sw01 * (step(0.0, gx01) - 0.5);
  gy01 -= sw01 * (step(0.0, gy01) - 0.5);

  vec4 gx10 = ixy10 / 7.0;
  vec4 gy10 = floor(gx10) / 7.0;
  vec4 gz10 = floor(gy10) / 6.0;
  gx10 = fract(gx10) - 0.5;
  gy10 = fract(gy10) - 0.5;
  gz10 = fract(gz10) - 0.5;
  vec4 gw10 = vec4(0.75) - abs(gx10) - abs(gy10) - abs(gz10);
  vec4 sw10 = step(gw10, vec4(0.0));
  gx10 -= sw10 * (step(0.0, gx10) - 0.5);
  gy10 -= sw10 * (step(0.0, gy10) - 0.5);

  vec4 gx11 = ixy11 / 7.0;
  vec4 gy11 = floor(gx11) / 7.0;
  vec4 gz11 = floor(gy11) / 6.0;
  gx11 = fract(gx11) - 0.5;
  gy11 = fract(gy11) - 0.5;
  gz11 = fract(gz11) - 0.5;
  vec4 gw11 = vec4(0.75) - abs(gx11) - abs(gy11) - abs(gz11);
  vec4 sw11 = step(gw11, vec4(0.0));
  gx11 -= sw11 * (step(0.0, gx11) - 0.5);
  gy11 -= sw11 * (step(0.0, gy11) - 0.5);

  vec4 g0000 = vec4(gx00.x,gy00.x,gz00.x,gw00.x);
  vec4 g1000 = vec4(gx00.y,gy00.y,gz00.y,gw00.y);
  vec4 g0100 = vec4(gx00.z,gy00.z,gz00.z,gw00.z);
  vec4 g1100 = vec4(gx00.w,gy00.w,gz00.w,gw00.w);
  vec4 g0010 = vec4(gx10.x,gy10.x,gz10.x,gw10.x);
  vec4 g1010 = vec4(gx10.y,gy10.y,gz10.y,gw10.y);
  vec4 g0110 = vec4(gx10.z,gy10.z,gz10.z,gw10.z);
  vec4 g1110 = vec4(gx10.w,gy10.w,gz10.w,gw10.w);
  vec4 g0001 = vec4(gx01.x,gy01.x,gz01.x,gw01.x);
  vec4 g1001 = vec4(gx01.y,gy01.y,gz01.y,gw01.y);
  vec4 g0101 = vec4(gx01.z,gy01.z,gz01.z,gw01.z);
  vec4 g1101 = vec4(gx01.w,gy01.w,gz01.w,gw01.w);
  vec4 g0011 = vec4(gx11.x,gy11.x,gz11.x,gw11.x);
  vec4 g1011 = vec4(gx11.y,gy11.y,gz11.y,gw11.y);
  vec4 g0111 = vec4(gx11.z,gy11.z,gz11.z,gw11.z);
  vec4 g1111 = vec4(gx11.w,gy11.w,gz11.w,gw11.w);

  vec4 norm00 = taylorInvSqrt(vec4(dot(g0000, g0000), dot(g0100, g0100), dot(g1000, g1000), dot(g1100, g1100)));
  g0000 *= norm00.x;
  g0100 *= norm00.y;
  g1000 *= norm00.z;
  g1100 *= norm00.w;

  vec4 norm01 = taylorInvSqrt(vec4(dot(g0001, g0001), dot(g0101, g0101), dot(g1001, g1001), dot(g1101, g1101)));
  g0001 *= norm01.x;
  g0101 *= norm01.y;
  g1001 *= norm01.z;
  g1101 *= norm01.w;

  vec4 norm10 = taylorInvSqrt(vec4(dot(g0010, g0010), dot(g0110, g0110), dot(g1010, g1010), dot(g1110, g1110)));
  g0010 *= norm10.x;
  g0110 *= norm10.y;
  g1010 *= norm10.z;
  g1110 *= norm10.w;

  vec4 norm11 = taylorInvSqrt(vec4(dot(g0011, g0011), dot(g0111, g0111), dot(g1011, g1011), dot(g1111, g1111)));
  g0011 *= norm11.x;
  g0111 *= norm11.y;
  g1011 *= norm11.z;
  g1111 *= norm11.w;

  float n0000 = dot(g0000, Pf0);
  float n1000 = dot(g1000, vec4(Pf1.x, Pf0.yzw));
  float n0100 = dot(g0100, vec4(Pf0.x, Pf1.y, Pf0.zw));
  float n1100 = dot(g1100, vec4(Pf1.xy, Pf0.zw));
  float n0010 = dot(g0010, vec4(Pf0.xy, Pf1.z, Pf0.w));
  float n1010 = dot(g1010, vec4(Pf1.x, Pf0.y, Pf1.z, Pf0.w));
  float n0110 = dot(g0110, vec4(Pf0.x, Pf1.yz, Pf0.w));
  float n1110 = dot(g1110, vec4(Pf1.xyz, Pf0.w));
  float n0001 = dot(g0001, vec4(Pf0.xyz, Pf1.w));
  float n1001 = dot(g1001, vec4(Pf1.x, Pf0.yz, Pf1.w));
  float n0101 = dot(g0101, vec4(Pf0.x, Pf1.y, Pf0.z, Pf1.w));
  float n1101 = dot(g1101, vec4(Pf1.xy, Pf0.z, Pf1.w));
  float n0011 = dot(g0011, vec4(Pf0.xy, Pf1.zw));
  float n1011 = dot(g1011, vec4(Pf1.x, Pf0.y, Pf1.zw));
  float n0111 = dot(g0111, vec4(Pf0.x, Pf1.yzw));
  float n1111 = dot(g1111, Pf1);

  vec4 fade_xyzw = fade(Pf0);
  vec4 n_0w = mix(vec4(n0000, n1000, n0100, n1100), vec4(n0001, n1001, n0101, n1101), fade_xyzw.w);
  vec4 n_1w = mix(vec4(n0010, n1010, n0110, n1110), vec4(n0011, n1011, n0111, n1111), fade_xyzw.w);
  vec4 n_zw = mix(n_0w, n_1w, fade_xyzw.z);
  vec2 n_yzw = mix(n_zw.xy, n_zw.zw, fade_xyzw.y);
  float n_xyzw = mix(n_yzw.x, n_yzw.y, fade_xyzw.x);
  return 2.2 * n_xyzw;
}

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

uniform vec4 u_hyperplane_normal;
uniform float u_hyperplane_displacement;

uniform mat4 u_transform;
uniform float u_time;

struct Tetrahedron
{
    vec4 vertices[4];
    vec4 cell_centroid;
};

struct Slice
{
    vec4 vertices[6];
};

struct DrawCommand
{
    uint count;
    uint instance_count;
    uint first;
    uint base_instance;
};

// Read only.
layout(std430, binding = 0) buffer BUFF_tetrahedra
{
    Tetrahedron tetrahedra[];
};

// Read + write.
layout(std430, binding = 1) buffer BUFF_slice_vertices
{
    Slice slice_vertices[];
};

layout(std430, binding = 2) buffer BUFF_indirect
{
    DrawCommand indirect[];
};

// Determined the signed distance between `point` and the hyperplane.
float side(in vec4 point)
{
    return dot(u_hyperplane_normal, point) + u_hyperplane_displacement;
}

// Clamp `value` between -1..1.
float saturate(float value)
{
    return min(1.0, max(-1.0, value));
}

void main()
{
    const uvec2 edge_indices[] =
    {
        { 0, 1 },
        { 0, 2 },
        { 0, 3 },
        { 1, 2 },
        { 1, 3 },
        { 2, 3 }
    };

    const uint max_intersections = 4;
    const uint max_new_vertices = 6;
    const uint ignore = 6;

    // Grab the appropriate tetrahedron based on this invocations local ID.
    uint local_id = gl_GlobalInvocationID.x;
    uint slice_id = 0;
    vec3 slice_centroid = vec3(0.0);
    Tetrahedron tetra = tetrahedra[local_id];

    // This array will be filled out with up to 4 unique points of intersection
    // in the for-loop below.
    vec4 intersections[4] =
    {
        vec4(0.0),
        vec4(0.0),
        vec4(0.0),
        vec4(0.0)
    };

    // Loop through all of this tetrahedron's edges.
    for (uint i = 0; i < edge_indices.length(); ++i)
    {
        uvec2 edge = edge_indices[i];
        vec4 a = u_transform * tetra.vertices[edge.x];
        vec4 b = u_transform * tetra.vertices[edge.y];

        float t = -side(a) / (side(b) - side(a));

        if (t >= 0.0 && t <= 1.0)
        {
            // Parallel projection from 4D -> 3D (drop the last coordinate);
            vec4 intersection = a + (b - a) * t;
            intersection = vec4(intersection.xyz, 1.0);

            // Store the point of intersection.
            intersections[slice_id] = intersection;

            slice_centroid += intersection.xyz;
            slice_id++;
        }
    }
    slice_centroid /= float(slice_id);

    // The variable `slice_id` is an integer corresponding to the number of valid
    // intersections that were found. Realistically, this should ONLY ever be
    // 0, 3, or 4.
    if (slice_id == 0) // Empty intersection (0-count draw call)
    {
        indirect[local_id] = DrawCommand(0, 0, local_id * max_new_vertices, 0);
    }
    else if (slice_id == 3) // Tri
    {
        slice_vertices[local_id].vertices[0] = intersections[0];
        slice_vertices[local_id].vertices[1] = intersections[1];
        slice_vertices[local_id].vertices[2] = intersections[2];

        // 3, 4, 5 are ignored...
        indirect[local_id] = DrawCommand(3, 1, local_id * max_new_vertices, 0);
    }
    else if (slice_id == 4) // Quad
    {
        // We have to use `vec2`s here instead of `uvec2`s because the signed angles
        // will be floating-point values.
        vec2 angles[max_intersections] =
        {
            { 0.0, 0.0 },
            { 1.0, 0.0 },
            { 2.0, 0.0 },
            { 3.0, 0.0 }
        };

        // Compute the slice normal (in 3-dimensions).
        vec3 a = intersections[0].xyz;
        vec3 b = intersections[1].xyz;
        vec3 c = intersections[2].xyz;
        vec3 ab = b - a;
        vec3 bc = c - b;
        vec3 n = normalize(cross(bc, ab));

        vec3 first_edge = normalize(a - slice_centroid);

        for (int i = 1; i < angles.length(); ++i)
        {
            vec3 p = intersections[i].xyz;
            vec3 edge = normalize(p - slice_centroid);

            float angle = saturate(dot(first_edge, edge));
            float signed_angle = acos(angle);

            if (dot(n, cross(first_edge, edge)) < 0.0)
            {
                signed_angle *= -1.0;
            }

            angles[i].y = signed_angle;
        }

        // Perform an insertion sort.
        uint i = 1;
        while(i < angles.length())
        {
            uint j = i;
            while(j > 0 && (angles[j - 1].y > angles[j].y))
            {
                vec2 temp = angles[j];
                angles[j] = angles[j - 1];
                angles[j - 1] = temp;

                j--;
            }
            i++;
        }

        // First triangle...(0, 1, 2)
        slice_vertices[local_id].vertices[0] = intersections[uint(angles[0].x)];
        slice_vertices[local_id].vertices[1] = intersections[uint(angles[1].x)];
        slice_vertices[local_id].vertices[2] = intersections[uint(angles[2].x)];

        // First triangle...(0, 2, 3)
        slice_vertices[local_id].vertices[3] = intersections[uint(angles[0].x)];
        slice_vertices[local_id].vertices[4] = intersections[uint(angles[2].x)];
        slice_vertices[local_id].vertices[5] = intersections[uint(angles[3].x)];

        indirect[local_id] = DrawCommand(max_new_vertices, 1, local_id * max_new_vertices, 0);
    }
    else
    {
        // We should never get here...
        indirect[local_id] = DrawCommand(0, 0, local_id * max_new_vertices, 0);
    }
}
