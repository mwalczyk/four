#version 450
#extension GL_ARB_shading_language_420pack : enable

uniform mat4 u_rotation;
uniform vec4 u_hyperplane_normal;
uniform float u_hyperplane_displacement;

struct tetrahedron
{
    vec4 vertices[4];
};

struct slice
{
    // The 4-th vertex might not be valid.
    vec3 vertices[4];
};

layout(std430, binding = 0) buffer tetrahedra
{
    tetrahedron simplices[];
};

layout(std430, binding = 1) buffer slices
{
    slice cross_sections[];
};

float side(in vec4 point)
{
    return dot(u_hyperplane_normal, point) + u_hyperplane_displacement;
}

void main()
{
    const int prim_restart = 65535;
    const ivec2 edge_indices[] =
    {
        { 0, 1 },
        { 0, 2 },
        { 0, 3 },
        { 1, 2 },
        { 1, 3 },
        { 2, 3 }
    };

    // Grab the appropriate tetrahedron based on this invocations local ID.
    int local_id = 0;
    int slice_id = 0;
    tetrahedron tetra = simplices[local_id];
    vec3 slice_centroid = vec3(0.0);

    // Loop through all of this tetrahedron's edges.
    for (int i = 0; i < edge_indices.length(); ++i)
    {
        ivec2 edge = edge_indices[i];
        vec4 a = u_rotation * tetra.vertices[edge.x];
        vec4 b = u_rotation * tetra.vertices[edge.y];

        float t = -side(a) / (side(b) - side(a));

        if (t >= 0.0 && t <= 1.0)
        {
            // Parallel projection from 4D -> 3D (drop the last coordinate);
            vec3 intersection = (a + (b - a) * t).xyz;
            slice_centroid += intersection;

            // Store the point of intersection.
            cross_sections[local_id * 4].vertices[slice_id] = intersection;
            slice_id++;
        }
    }
    slice_centroid /= float(slice_id);

    // Sort points...
    for (int i = 0; i < 4; ++i)
    {

    }
}
