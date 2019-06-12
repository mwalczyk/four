#version 450

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

uniform vec4 u_hyperplane_normal;
uniform float u_hyperplane_displacement;

uniform mat4 u_transform;
uniform float u_time;

struct Tetrahedron
{
    vec4 vertices[4];
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
            //intersection = vec4(intersection.xyz, 1.0);

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

        // TODO: determine outward facing normal `https://www.gamedev.net/forums/topic/433315-determining-outward-facing-normals/?do=findComment&comment=3880903`

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
