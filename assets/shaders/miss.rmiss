#version 460
#extension GL_NV_ray_tracing : require

layout(location = 0) rayPayloadInNV vec3 hitValue;
layout(binding = 7, set = 0) uniform ClearColor { vec4 clear; } clearColor;

void main()
{
    hitValue = clearColor.clear.xyz;
}