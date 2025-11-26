#version 100
precision lowp float;

varying vec2 fragTexCoord;
varying vec4 fragColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;

uniform vec4 original_0;
uniform vec4 original_1;

uniform vec4 replace_0;
uniform vec4 replace_1;

// Tool color swap uniforms (only used when mining)
uniform vec4 tool_original_primary;
uniform vec4 tool_original_secondary;
uniform vec4 tool_replace_primary;
uniform vec4 tool_replace_secondary;
uniform float tool_swap_enabled;

uniform float exhustion;
uniform float whiteout;

const vec3 skin = vec3(255.0/255.0, 174.0/255.0, 112.0/255.0);
const vec3 skin2 = vec3(204.0/255.0, 147.0/255.0, 98.0/255.0);

void main() {
	vec4 tex = texture2D(texture0, fragTexCoord);
	vec4 color;

	if (whiteout > 0.5) {
		color = vec4(1.0, 1.0, 1.0, tex.a);
	} else if (distance(tex, original_0) <= 0.01) {
		color = replace_0;
	} else if (distance(tex, original_1) <= 0.01) {
		color = replace_1;
	} else if (tool_swap_enabled > 0.5 && distance(tex, tool_original_primary) <= 0.01) {
		color = tool_replace_primary;
	} else if (tool_swap_enabled > 0.5 && distance(tex, tool_original_secondary) <= 0.01) {
		color = tool_replace_secondary;
	} else if (distance(tex, vec4(skin, 1.0)) <= 0.01) {
		color = vec4(mix(skin, vec3(1.0, 0.0, 0.0), exhustion / 2.0), 1.0);
	} else if (distance(tex, vec4(skin2, 1.0)) <= 0.01) {
		color = vec4(mix(skin2, vec3(1.0, 0.0, 0.0), exhustion / 2.0), 1.0);
	} else {
		color = tex;
	}

	gl_FragColor = color * colDiffuse * fragColor;
}
