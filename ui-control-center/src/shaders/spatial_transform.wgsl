// spatial_transform.wgsl
// WebGPU Compute Shader — Coordinate Transformation Pipeline
// Purpose: Parallel projection of 3D spatial coordinates to 2D viewport
//          coordinates with LoD-based point culling.
// Target: wasm32 + WebGPU (wgpu crate)
//
// Dispatch: (ceil(N / 256), 1, 1) workgroups of 256 threads each
//           where N = total point count

struct Point {
    lat: f32,
    lon: f32,
    alt: f32,
}

struct ViewportTransform {
    center_lat: f32,
    center_lon: f32,
    zoom: f32,
    screen_width: f32,
    screen_height: f32,
    lod_threshold: f32,
}

struct OutputPoint {
    screen_x: f32,
    screen_y: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    visible: u32,
}

// Storage buffers bound from Wasm memory heap
@group(0) @binding(0) var<storage, read> input_points: array<Point>;
@group(0) @binding(1) var<storage, read> viewport: ViewportTransform;
@group(0) @binding(2) var<storage, read_write> output_points: array<OutputPoint>;

// Color mapping: altitude → warm (low) to cool (high)
fn altitude_to_color(alt: f32, min_alt: f32, max_alt: f32) -> vec3<f32> {
    let range = max(1.0, max_alt - min_alt);
    let t = clamp((alt - min_alt) / range, 0.0, 1.0);

    // Low altitude: warm orange/red
    // High altitude: cool blue/purple
    if t < 0.33 {
        return mix(vec3<f32>(0.98, 0.32, 0.20), vec3<f32>(0.82, 0.60, 0.24), t * 3.0);
    } else if t < 0.66 {
        return mix(vec3<f32>(0.82, 0.60, 0.24), vec3<f32>(0.35, 0.74, 0.35), (t - 0.33) * 3.0);
    } else {
        return mix(vec3<f32>(0.35, 0.74, 0.35), vec3<f32>(0.49, 0.36, 0.96), (t - 0.66) * 3.0);
    }
}

// Mercator projection: (lat, lon) → (x, y) in normalized device coords
fn mercator_project(lat: f32, lon: f32, center_lat: f32, center_lon: f32, zoom: f32) -> vec2<f32> {
    // Convert to radians
    let lat_rad = radians(lat);
    let lon_rad = radians(lon);
    let center_lat_rad = radians(center_lat);

    // Mercator y-coordinate
    let y = log(tan(PI / 4.0 + lat_rad / 2.0));
    let center_y = log(tan(PI / 4.0 + center_lat_rad / 2.0));

    // Scale and offset
    let scale = zoom * 256.0 / (2.0 * PI);
    let x = (lon_rad - radians(center_lon)) * scale;
    let dy = (y - center_y) * scale;

    return vec2<f32>(x, dy);
}

// Level-of-Detail culling: return false if point should be rendered
fn lod_visible(point: Point, center_lat: f32, center_lon: f32, zoom: f32, lod: f32) -> bool {
    // At low zoom (zoomed out), skip low-altitude points (they cluster)
    // At high zoom (zoomed in), show all points
    let distance_from_center = sqrt(
        (point.lat - center_lat) * (point.lat - center_lat) +
        (point.lon - center_lon) * (point.lon - center_lon)
    );

    // Cull points whose spatial frequency exceeds the LoD threshold
    return distance_from_center < lod * 5.0;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    // Bounds check — guard against out-of-bounds dispatch
    if idx >= arrayLength(&input_points) {
        return;
    }

    let point = input_points[idx];

    // LoD culling
    if !lod_visible(point, viewport.center_lat, viewport.center_lon, viewport.zoom, viewport.lod_threshold) {
        output_points[idx].visible = 0u;
        return;
    }

    // Project to screen coordinates
    let projected = mercator_project(
        point.lat, point.lon,
        viewport.center_lat, viewport.center_lon,
        viewport.zoom,
    );

    // Convert to screen pixels
    let screen_x = viewport.screen_width / 2.0 + projected.x;
    let screen_y = viewport.screen_height / 2.0 - projected.y;

    // Color mapping
    let color = altitude_to_color(point.alt, 0.0, 12000.0);

    // Write output
    output_points[idx] = OutputPoint(
        screen_x,
        screen_y,
        color.r,
        color.g,
        color.b,
        1u,
    );
}
