#version 440
// The poster inside the tile's rounded rect: a signed distance to the rect gives each pixel
// its coverage, so the corner is antialiased over exactly one pixel and nothing under the
// card can show through a soft edge.
layout(location = 0) in vec2 texCoord;
layout(location = 1) in vec2 local;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float radius;
    vec2 size;
    vec4 subRect;
};
layout(binding = 1) uniform sampler2D source;
void main()
{
    vec2 halfSize = size * 0.5;
    vec2 q = abs(local - halfSize) - (halfSize - vec2(radius));
    float d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
    float coverage = clamp(0.5 - d, 0.0, 1.0);
    // Inside the texture's own part of an atlas, so the pixel beyond the tile's edge
    // repeats the poster's border rather than a neighbour's
    vec2 uv = clamp(texCoord, subRect.xy, subRect.zw);
    fragColor = texture(source, uv) * (coverage * qt_Opacity);
}
