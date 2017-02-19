#version 400
uniform mat4 view_matrix;
uniform mat4 proj_matrix;

out float z;
out float dist;
out vec3 world_pos;

in vec3 position;
in float distance_from_surface;

void main() {
    z = position.z;
    dist = distance_from_surface;
    world_pos = position;

    gl_Position = vec4(
        proj_matrix
            * view_matrix
            * vec4(position, 1.0)
    );
}
