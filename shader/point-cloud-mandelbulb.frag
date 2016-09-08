#version 140

in vec3 ocolor;
in float z;

out vec4 color;

void main() {
    color = vec4(ocolor, 1.0);
}
