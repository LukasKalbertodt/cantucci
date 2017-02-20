float shape_de(vec3 point) {
    return length(vec3({X}, {Y}, {Z}) - point) - {RADIUS};
}
