#version 100

precision mediump float;

varying vec2 fragTexCoord;

uniform sampler2D texture0;    // Raylib automatically binds texture to texture0
uniform float ambientDarkness; // Base darkness from depth
uniform vec2 textureSize;      // Size of texture in pixels

void main() {
    // Calculate position in tile space
    vec2 tileCoord = fragTexCoord * textureSize;
    
    // Find the 4 nearest tile centers for bilinear interpolation
    vec2 baseTile = floor(tileCoord - 0.5);
    vec2 blend = fract(tileCoord - 0.5);
    
    // Sample the 4 tile centers
    vec4 sample00 = texture2D(texture0, (baseTile + vec2(0.5, 0.5)) / textureSize);
    vec4 sample10 = texture2D(texture0, (baseTile + vec2(1.5, 0.5)) / textureSize);
    vec4 sample01 = texture2D(texture0, (baseTile + vec2(0.5, 1.5)) / textureSize);
    vec4 sample11 = texture2D(texture0, (baseTile + vec2(1.5, 1.5)) / textureSize);
    
    // Check which of the 4 tiles are air (G channel = is_solid, so invert it)
    float air00 = 1.0 - step(0.5, sample00.g);
    float air10 = 1.0 - step(0.5, sample10.g);
    float air01 = 1.0 - step(0.5, sample01.g);
    float air11 = 1.0 - step(0.5, sample11.g);
    
    // Calculate weighted average, only including air tiles
    // Weight each sample by: (bilinear weight) * (is_air)
    float w00 = (1.0 - blend.x) * (1.0 - blend.y) * air00;
    float w10 = blend.x * (1.0 - blend.y) * air10;
    float w01 = (1.0 - blend.x) * blend.y * air01;
    float w11 = blend.x * blend.y * air11;
    
    float totalWeight = w00 + w10 + w01 + w11;
    
    float brightness;
    if (totalWeight > 0.01) {
        // Weighted average of air tiles only
        brightness = (sample00.r * w00 + sample10.r * w10 + sample01.r * w01 + sample11.r * w11) / totalWeight;
    } else {
        // All surrounding tiles are solid, use nearest
        vec2 nearestTile = floor(tileCoord);
        brightness = texture2D(texture0, (nearestTile + vec2(0.5, 0.5)) / textureSize).r;
    }
    
    // Calculate final darkness: ambient darkness minus local brightness
    // Light reduces the ambient darkness, but can't make it negative
    float darkness = max(ambientDarkness - brightness, 0.0);
    
    // Output pure black with alpha = darkness
    gl_FragColor = vec4(0.0, 0.0, 0.0, darkness);
}
