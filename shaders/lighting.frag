#version 100

precision mediump float;

varying vec2 fragTexCoord;

uniform sampler2D texture0;    // Raylib automatically binds texture to texture0
uniform float ambientDarkness; // Base darkness from depth

void main() {
    // Sample lighting texture (GPU bilinear filtering provides smoothness)
    // fragTexCoord is already in correct UV space (0-1)
    float brightness = texture2D(texture0, fragTexCoord).r;
    
    // Calculate final darkness: ambient darkness minus local brightness
    // Light reduces the ambient darkness, but can't make it negative
    float darkness = max(ambientDarkness - brightness, 0.0);
    
    // Output pure black with alpha = darkness
    gl_FragColor = vec4(0.0, 0.0, 0.0, darkness);
}
