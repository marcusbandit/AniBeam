#version 440
// The poster card's picture (CardFace, #289): a quad a pixel larger than the tile on every
// side, so the fragment stage can fade the rounded edge over its last pixel.
layout(location = 0) in vec4 vertexPosition;
layout(location = 1) in vec2 vertexTexCoord;
layout(location = 2) in vec2 vertexLocal;
layout(location = 0) out vec2 texCoord;
layout(location = 1) out vec2 local;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float radius;
    vec2 size;
    vec4 subRect;
};
out gl_PerVertex { vec4 gl_Position; };
void main()
{
    texCoord = vertexTexCoord;
    local = vertexLocal;
    gl_Position = qt_Matrix * vertexPosition;
}
