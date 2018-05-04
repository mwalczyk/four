#version 430
#extension GL_ARB_shading_language_420pack : enable

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

uniform vec4 u_hyperplane_normal;
uniform float u_hyperplane_displacement;
uniform mat4 u_rotation;

struct Tetrahedron
{
    vec4 vertices[4];
};

struct VertGroup
{
    vec4 vertices[4];
};

// Read only.
layout(std430, binding = 0) buffer BUFF_tetrahedra
{
    Tetrahedron tetrahedra[3240];
};

// Read + write.
layout(std430, binding = 1) buffer BUFF_slice_vertices
{
    VertGroup slice_vertices[3240];
};

layout(std430, binding = 2) buffer BUFF_slice_indices
{
    uvec4 slice_indices[3240];
};

float side(in vec4 point)
{
    return dot(u_hyperplane_normal, point) + u_hyperplane_displacement;
}

float saturate(float value)
{
    return min(1.0, max(-1.0, value));
}

void main()
{
    const uint prim_restart = 65535; // 0xFFFF
    const uvec2 edge_indices[] =
    {
        { 0, 1 },
        { 0, 2 },
        { 0, 3 },
        { 1, 2 },
        { 1, 3 },
        { 2, 3 }
    };

    // Grab the appropriate tetrahedron based on this invocations local ID.
    uint local_id = gl_GlobalInvocationID.x;
    uint slice_id = 0;
    Tetrahedron tetra = tetrahedra[local_id];
    vec3 slice_centroid = vec3(0.0);

    if (local_id > 3239)
    {
        return;
    }
    // Loop through all of this tetrahedron's edges.
    for (uint i = 0; i < edge_indices.length(); ++i)
    {
        uvec2 edge = edge_indices[i];
        vec4 a = u_rotation * tetra.vertices[edge.x];
        vec4 b = u_rotation * tetra.vertices[edge.y];

        float t = -side(a) / (side(b) - side(a));

        if (t >= 0.0 && t <= 1.0)
        {
            // Parallel projection from 4D -> 3D (drop the last coordinate);
            vec4 intersection = a + (b - a) * t;
            intersection = vec4(intersection.xyz, 1.0);

            slice_centroid += intersection.xyz;

            // Store the point of intersection.
            slice_vertices[local_id].vertices[slice_id] = intersection;

            slice_id++;
        }
    }
    slice_centroid /= float(slice_id);

    // The variable `slice_id` is an integer corresponding to the number of valid
    // intersections that were found. Realistically, this should ONLY ever be
    // 0, 1, 2, 3 or 4.
    if (slice_id == 0)
    {
        slice_indices[local_id] = uvec4(prim_restart);

        slice_vertices[local_id].vertices[0] = vec4(0.0);
        slice_vertices[local_id].vertices[1] = vec4(0.0);
        slice_vertices[local_id].vertices[2] = vec4(0.0);
        slice_vertices[local_id].vertices[3] = vec4(0.0);
    }
    else if (slice_id == 3)
    {
        // Tri
        slice_indices[local_id] = uvec4(
            0 + local_id * 4,
            1 + local_id * 4,
            2 + local_id * 4,
            prim_restart
        );

        slice_vertices[local_id].vertices[3] = vec4(0.0);
    }
    else if (slice_id == 4)
    {
        // Quad
        vec2 angles[4] =
        {
            vec2(0.0, 0.0),
            vec2(1.0, 0.0),
            vec2(2.0, 0.0),
            vec2(3.0, 0.0)
        };

        vec3 a = slice_vertices[local_id].vertices[0].xyz;
        vec3 b = slice_vertices[local_id].vertices[1].xyz;
        vec3 c = slice_vertices[local_id].vertices[2].xyz;

        vec3 ab = b - a;
        vec3 bc = c - b;
        vec3 n = normalize(cross(bc, ab));
        vec3 first_edge = normalize(a - slice_centroid);

        for (int i = 1; i < 4; ++i)
        {
            vec3 p = slice_vertices[local_id].vertices[i].xyz;
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
        while(i < 4)
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

        slice_indices[local_id] = uvec4(
            uint(angles[0].x) + local_id * 4,
            uint(angles[1].x) + local_id * 4,
            uint(angles[3].x) + local_id * 4,
            uint(angles[2].x) + local_id * 4
        );
    }
    else
    {
        // TODO: we should never get here...I don't think?
        slice_indices[local_id] = uvec4(prim_restart);
    }
}
