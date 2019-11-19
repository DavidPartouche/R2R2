#version 460
#extension GL_NV_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) rayPayloadInNV vec3 hitValue;
layout(location = 2) rayPayloadNV bool isShadowed;

hitAttributeNV vec3 attribs;
layout(binding = 0, set = 0) uniform accelerationStructureNV topLevelAS;
layout(binding = 3, set = 0) buffer Vertices { vec3 v[]; }
vertices;
layout(binding = 4, set = 0) buffer Indices { uint i[]; }
indices;
layout(binding = 5, set = 0) buffer MatColorBufferObject { vec4[] m; }
materials;
layout(binding = 6, set = 0) uniform sampler2D[] textureSamplers;

struct Vertex {
    vec3 pos;
    vec3 nrm;
//    vec3 color;
//    vec2 texCoord;
//    int matIndex;
};

uint vertexSize = 3;

Vertex unpackVertex(uint index) {
    Vertex v;
    vec3 d0 = vertices.v[vertexSize * index + 0];
    vec3 d1 = vertices.v[vertexSize * index + 1];
//    vec4 d2 = vertices.v[vertexSize * index + 2];

    v.pos = d0;
    v.nrm = d1;
//    v.pos = d0.xyz;
//    v.nrm = vec3(d0.w, d1.x, d1.y);
//    v.color = vec3(d1.z, d1.w, d2.x);
//    v.texCoord = vec2(d2.y, d2.z);
//    v.matIndex = floatBitsToInt(d2.w);
    return v;
}

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    vec3 transmittance;
    vec3 emission;
    float shininess;
    float ior;
    float dissolve;
    int illum;
    int textureId;
};

const int matSize = 5;

Material unpackMaterial(int matIndex) {
    Material m;
    vec4 d0 = materials.m[matSize * matIndex + 0];
    vec4 d1 = materials.m[matSize * matIndex + 1];
    vec4 d2 = materials.m[matSize * matIndex + 2];
    vec4 d3 = materials.m[matSize * matIndex + 3];
    vec4 d4 = materials.m[matSize * matIndex + 4];

    m.ambient = d0.xyz;
    m.diffuse = vec3(d0.w, d1.x, d1.y);
    m.specular = vec3(d1.z, d1.w, d2.x);
    m.transmittance = vec3(d2.y, d2.z, d2.w);
    m.emission = d3.xyz;
    m.shininess = d3.w;
    m.ior = d4.x;
    m.dissolve = d4.y;
    m.illum = int(d4.z);
    m.textureId = floatBitsToInt(d4.w);
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

   vec3 c = dot_product * vec3(0.7, 0.7, 0.7);
    /*Material mat = unpackMaterial(v1.matIndex);
    vec3 c = dot_product * mat.diffuse;
    if(mat.textureId >= 0) {
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
