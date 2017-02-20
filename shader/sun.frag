#version 140

uniform float day_theta;

in vec2 x_unit_pos;

out vec4 o_color;

void main() {
    float r = x_unit_pos.x * x_unit_pos.x + x_unit_pos.y * x_unit_pos.y;
    if (r > 1) {
        discard;
    } else {
        // Make the edges of the sun soft
        float a = (1 - r) * 4 - 1;

        // We modify the alpha value to slightly animate the edges of the sun
        // as well as making them "rough".
        float angle = atan(x_unit_pos.y, x_unit_pos.x);
        a -= 0.01 * sin(angle * 41)
            + 0.03 * cos(-angle * 13 + 50 * day_theta)
            + 0.04 * sin(angle * 7 + 30 * day_theta);

        o_color = vec4(1, 0.95, 0.07, a);
    }
}
