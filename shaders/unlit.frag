#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 fragColor;

layout(binding = 1) uniform Animation {
    float anim;
};

layout(binding = 2) uniform sampler2D tex;

layout(location = 0) out vec4 outColor;

void main() {
    vec2 uv = fragColor.xy;
    //uv.x += cos(anim * 10. + uv.y * 50.) * .2;
    outColor = texture(tex, uv);
}
