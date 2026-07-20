#version 100
precision mediump float;

void main() {
    vec2 pos = gl_FragCoord.xy;
    float grid_size = 20.0;
    vec3 color = vec3(0.);
    color = vec3(0.13,0.13,0.13);
    float offset = 0.0;

    // offset every other line (so we get grid and not stripes)
    if (mod(pos.y/grid_size,2.0) < 1.0) {
        offset = 1.0;
    }
    // change color of every other pixel
    if (mod(pos.x/grid_size+offset,2.0) < 1.0) {
        color = vec3(0.25,0.25,0.25);
    }
    gl_FragColor = vec4(color,1.0);
}