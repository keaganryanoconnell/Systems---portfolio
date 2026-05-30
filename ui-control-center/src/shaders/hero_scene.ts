// Vertex shader — full-screen quad for raymarching
export const heroSceneVert = /* glsl */ `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = vec4(position, 1.0);
}
`;

// Fragment shader — raymarched procedural scene
export const heroSceneFrag = /* glsl */ `
uniform float uTime;
uniform vec2 uMouse;
uniform float uScroll;
uniform vec2 uResolution;
varying vec2 vUv;

#define GOLD vec3(0.824, 0.600, 0.114)
#define BLUE vec3(0.345, 0.651, 1.0)
#define BG vec3(0.035, 0.047, 0.063)
#define PI 3.14159265

float smin(float a, float b, float k) {
  float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
  return mix(b, a, h) - k * h * (1.0 - h);
}

float sdSphere(vec3 p, float r) { return length(p) - r; }
float sdTorus(vec3 p, vec2 t) {
  vec2 q = vec2(length(p.xz) - t.x, p.y);
  return length(q) - t.y;
}
float sdBox(vec3 p, vec3 b) {
  vec3 q = abs(p) - b;
  return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}
float sdOctahedron(vec3 p, float s) {
  p = abs(p);
  return (p.x + p.y + p.z - s) * 0.57735027;
}

mat3 rotY(float a) {
  float s = sin(a), c = cos(a);
  return mat3(c, 0, -s, 0, 1, 0, s, 0, c);
}
mat3 rotX(float a) {
  float s = sin(a), c = cos(a);
  return mat3(1, 0, 0, 0, c, -s, 0, s, c);
}

float map(vec3 p) {
  float t = uTime * 0.3;
  float scroll = uScroll * 3.0;

  // Floating shapes orbit
  vec3 p1 = p;
  p1.xz *= rotY(t * 0.7 + scroll).xz;
  p1 -= vec3(1.2, 0.0, 0.0);
  float d1 = sdTorus(p1, vec2(0.35, 0.08));

  vec3 p2 = p;
  p2.xz *= rotY(-t * 0.5 + scroll * 0.7).xz;
  p2 -= vec3(-0.9, 0.3, 0.8);
  float d2 = sdOctahedron(p2, 0.3);

  vec3 p3 = p;
  p3.xz *= rotY(t * 0.6 - scroll * 0.5).xz;
  p3 -= vec3(0.5, -0.2, -0.6);
  float d3 = sdBox(p3, vec3(0.25, 0.18, 0.25));

  vec3 p4 = p;
  p4.xz *= rotY(-t * 0.4).xz;
  p4 -= vec3(-0.5, -0.15, 1.0);
  float d4 = sdSphere(p4, 0.22);

  // Invisible ground
  float ground = p.y + 2.5;

  float d = d1;
  d = smin(d, d2, 0.35);
  d = smin(d, d3, 0.3);
  d = smin(d, d4, 0.3);
  d = min(d, ground);

  return d;
}

vec3 calcNormal(vec3 p) {
  float h = 0.0001;
  vec2 k = vec2(1, -1);
  return normalize(
    k.xyy * map(p + k.xyy * h) +
    k.yyx * map(p + k.yyx * h) +
    k.yxy * map(p + k.yxy * h) +
    k.xxx * map(p + k.xxx * h)
  );
}

float softShadow(vec3 ro, vec3 rd, float maxDist, float k) {
  float res = 1.0;
  float t = 0.02;
  for (int i = 0; i < 32; i++) {
    float h = map(ro + rd * t);
    if (h < 0.001) return 0.0;
    res = min(res, k * h / t);
    t += h;
  }
  return res;
}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * uResolution) / uResolution.y;

  float mx = uMouse.x * 0.3;
  float my = uMouse.y * 0.2;

  vec3 ro = vec3(mx * 1.5, my * 1.2, 4.5);
  vec3 lookAt = vec3(0.0, 0.0, 0.0);

  vec3 fwd = normalize(lookAt - ro);
  vec3 right = normalize(cross(fwd, vec3(0, 1, 0)));
  vec3 up = cross(right, fwd);

  vec3 rd = normalize(fwd + right * uv.x * 1.2 + up * uv.y * 1.2);

  float t = 0.0;
  float maxDist = 10.0;
  bool hit = false;

  for (int i = 0; i < 80; i++) {
    float d = map(ro + rd * t);
    if (d < 0.001) { hit = true; break; }
    t += d * 0.7;
    if (t > maxDist) break;
  }

  vec3 col = BG;

  if (hit) {
    vec3 pos = ro + rd * t;
    vec3 n = calcNormal(pos);

    float ao = softShadow(pos, n, 3.0, 8.0);

    vec3 lightDir = normalize(vec3(0.5, 0.8, 0.6));
    float diff = max(dot(n, lightDir), 0.0);
    float fresnel = pow(1.0 - abs(dot(n, rd)), 4.0);

    vec3 matCol = mix(GOLD, BLUE, sin(pos.x * 4.0) * 0.5 + 0.5);
    matCol = mix(matCol, GOLD, fresnel * 0.6);

    col = matCol * (diff * 0.7 + 0.3 * ao);
    col += GOLD * fresnel * 0.4;
  } else {
    // Background grid
    float grid = abs(fract(uv.x * 20.0 + uScroll * 0.5) - 0.5);
    grid = min(grid, abs(fract(uv.y * 20.0) - 0.5));
    float gridLine = 1.0 - smoothstep(0.0, 0.02, grid);
    col += vec3(0.35, 0.65, 1.0) * gridLine * 0.05;
  }

  // Vignette
  float vignette = 1.0 - dot(uv, uv) * 0.4;
  col *= vignette;

  gl_FragColor = vec4(col, 1.0);
}
`;
