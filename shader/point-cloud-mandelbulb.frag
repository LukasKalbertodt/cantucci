#version 140

in vec3 ocolor;
in float z;
in vec3 world_pos;

out vec4 color;

void main() {
    float c = abs(1.0 - length(world_pos)) * 1000;

    color = vec4(ocolor, 1.0);
}
