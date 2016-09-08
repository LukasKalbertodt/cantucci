#version 400
uniform dmat4 view_matrix;
uniform dmat4 proj_matrix;

out float z;
out vec3 ocolor;

in vec3 position;
in vec3 color;

void main() {
    z = position.z;
    ocolor = color;

    gl_Position = vec4(
        proj_matrix *
        view_matrix *
        vec4(position, 1.0)
    );
}
