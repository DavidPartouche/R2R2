#version 460
#extension GL_NV_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) rayPayloadInNV vec3 hitValue;
layout(location = 2) rayPayloadNV bool isShadowed;

hitAttributeNV vec3 attribs;
layout(binding = 0, set = 0) uniform accelerationStructureNV topLevelAS;
layout(binding = 3, set = 0) buffer Vertices { vec4 v[]; }
vertices;
layout(binding = 4, set = 0) buffer Indices { uint i[]; }
indices;
layout(binding = 5, set = 0) buffer MatColorBufferObject { vec4[] m; }
materials;
layout(binding = 6, set = 0) uniform sampler2D[] textureSamplers;

struct Vertex {
    vec3 pos;
    vec3 nrm;
    vec2 texCoord;
};

uint vertexSize = 2;

Vertex unpackVertex(uint index) {
    Vertex v;
    vec4 d0 = vertices.v[vertexSize * index + 0];
    vec4 d1 = vertices.v[vertexSize * index + 1];
    v.pos = d0.xyz;
    v.nrm = vec3(d0.w, d1.x, d1.y);
    v.texCoord = vec2(d1.z, d1.w);
    return v;
}

struct Material {
    vec4 baseColorFactor;
    float metallicFactor;
    float roughnessFactor;
};

const int matSize = 2;

Material unpackMaterial(int matIndex) {
    Material m;
    m.baseColorFactor = materials.m[matSize * matIndex + 0];
    vec4 d = materials.m[matSize * matIndex + 1];
    m.metallicFactor = d.x;
    m.roughnessFactor = d.y;
    return m;
}

void main()
{
    ivec3 ind = ivec3(indices.i[3 * gl_PrimitiveID], indices.i[3 * gl_PrimitiveID + 1],
    indices.i[3 * gl_PrimitiveID + 2]);

    Vertex v0 = unpackVertex(ind.x);
    Vertex v1 = unpackVertex(ind.y);
    Vertex v2 = unpackVertex(ind.z);

    const vec3 barycentrics = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);
    vec3 normal = normalize(v0.nrm * barycentrics.x + v1.nrm * barycentrics.y + v2.nrm * barycentrics.z);

    vec3 lightVector = normalize(vec3(5, 4, 3));
    float dot_product = max(dot(lightVector, normal), 0.2);

    Material mat = unpackMaterial(0);
    vec3 c = dot_product * mat.baseColorFactor.xyz;
    /*if(mat.textureId >= 0) {
        vec2 texCoord = v0.texCoord * barycentrics.x + v1.texCoord * barycentrics.y + v2.texCoord * barycentrics.z;
        c *= texture(textureSamplers[mat.textureId], texCoord).xyz;
    }*/

    float tmin = 0.001;
    float tmax = 100.0;
    vec3 origin = gl_WorldRayOriginNV + gl_WorldRayDirectionNV * gl_HitTNV;
    isShadowed = true;
    traceNV(topLevelAS, gl_RayFlagsTerminateOnFirstHitNV|gl_RayFlagsOpaqueNV|gl_RayFlagsSkipClosestHitShaderNV, 0xFF, 1, 0, 1, origin, tmin, lightVector, tmax, 2);

    if (isShadowed) {
        hitValue = c * 0.3;
    }
    else {
        hitValue = c;
    }
}
