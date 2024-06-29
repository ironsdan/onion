#version 450

layout(location = 0) in vec3 v_color;

layout(push_constant) uniform constants {
	vec4 data;
	mat4 render_matrix;
} pc;

layout(location = 0) out vec4 f_color;

void main() {
    // vec2 point = pc.data.xy;
    // float d = distance(point, vec2(gl_FragCoord.x/pc.data.z, gl_FragCoord.y/pc.data.w));    
    // f_color = vec4(vec3(d), 1.0);
    f_color = vec4(v_color, 1.0);
}