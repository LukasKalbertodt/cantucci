#version 140

// INCLUDE(DE)

in vec3 world_pos;

out vec3 color;

#define PI 3.1415926

void main() {
    // Calculate spherical coordinates
    vec3 unit = normalize(world_pos);
    float theta = acos(unit.z);         // z in [-1...1] => theta in [PI..0]
    float thn = theta / PI;
    float phi = atan(unit.y, unit.x);     // Phi is in [0..2PI]

    // Color definitions
    vec3 horizon_blue = vec3(0.03, 0.45, 0.9);
    vec3 top_blue = vec3(0.0, 0.1, 0.42);
    vec3 buttom_grey = vec3(0.2, 0.1, 0.25);

    if (thn <= 0.5) {
        // Upper hemisphere
        color = mix(top_blue, horizon_blue, pow(thn * 2, 2));
    } else if (thn <= 0.55) {
        color = horizon_blue;
    } else {
        color = mix(buttom_grey, horizon_blue, pow((1.05 - thn) * 2, 4));
    }
    color *= 1 - 0.3 * sin(phi) * sin(theta);


}
