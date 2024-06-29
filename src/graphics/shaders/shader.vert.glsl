#version 460

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 color;

layout(push_constant) uniform constants {
	vec4 data;
	mat4 render_matrix;
} pc;

layout (location = 0) out vec3 out_color;

void main() {
	vec4 t = pc.render_matrix * vec4(position, 1.0f);
	gl_Position = t;
	out_color = color;
}