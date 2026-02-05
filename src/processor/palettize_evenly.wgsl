@group(0) @binding(0) var<uniform> dimensions: vec4<u32>;
@group(0) @binding(1) var<storage, read> pixels: array<u32>;
@group(0) @binding(2) var<storage, read> palette: array<u32>;
@group(0) @binding(3) var<storage, read_write> shader_results: array<u32>;

fn unpack(color: u32) -> vec3<f32> {
    return vec3<f32>(
        f32(color & 0xFF),
        f32((color >> 8) & 0xFF),
        f32((color >> 16) & 0xFF),
    );
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width = dimensions.x;
    let height = dimensions.y;
    let palette_length = dimensions.w;

    let x = global_id.x;
    let y = global_id.y;

    if (x >= width || y >= height) {
        return;
    }

    let pixel_index = (y * width) + x;
    let pixel = unpack(pixels[pixel_index]);

    var closest_color_distance = 999999.0;
    var closest_color_index = 0u;

    for (var i = 0u; i < palette_length; i++) {
        let color = unpack(palette[i]);

        let r_value = (pixel.x - color.x) * 0.299;
        let g_value = (pixel.y - color.y) * 0.587;
        let b_value = (pixel.z - color.z) * 0.114;
        let distance = sqrt((r_value * r_value) + (g_value * g_value) + (b_value * b_value));

        if (distance < closest_color_distance) {
            closest_color_distance = distance;
            closest_color_index = i;
        }
    }

    shader_results[pixel_index] = closest_color_index;
}