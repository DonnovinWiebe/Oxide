@group(0) @binding(0) var<uniform> dimensions: vec4<u32>;
@group(0) @binding(1) var<storage, read> pixels: array<u32>;
@group(0) @binding(2) var<storage, read> biased_pallet: array<u32>;
@group(0) @binding(3) var<storage, read> standard_pallet: array<u32>;
@group(0) @binding(4) var<storage, read_write> shader_results: array<u32>;

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
    let biased_pallet_length = dimensions.z;
    let standard_pallet_length = dimensions.w;

    let x = global_id.x;
    let y = global_id.y;

    if (x >= width || y >= height) {
        return;
    }

    let pixel_index = (y * width) + x;
    let pixel = unpack(pixels[pixel_index]);

    var closest_biased_color_distance = 999999.0;
    var closest_biased_color_index = 0u;

    for (var i = 0u; i < biased_pallet_length; i++) {
        let color = unpack(biased_pallet[i]);

        let r_value = (pixel.x - color.x) * 0.299 / 1.5;
        let g_value = (pixel.y - color.y) * 0.587 / 1.5;
        let b_value = (pixel.z - color.z) * 0.114 / 1.5;
        let distance = sqrt((r_value * r_value) + (g_value * g_value) + (b_value * b_value));

        if (distance < closest_biased_color_distance) {
            closest_biased_color_distance = distance;
            closest_biased_color_index = i;
        }
    }

    var closest_standard_color_distance = 999999.0;
    var closest_standard_color_index = 0u;

    for (var i = 0u; i < standard_pallet_length; i++) {
        let color = unpack(standard_pallet[i]);

        let r_value = (pixel.x - color.x) * 0.299;
        let g_value = (pixel.y - color.y) * 0.587;
        let b_value = (pixel.z - color.z) * 0.114;
        let distance = sqrt((r_value * r_value) + (g_value * g_value) + (b_value * b_value));

        if (distance < closest_standard_color_distance) {
            closest_standard_color_distance = distance;
            closest_standard_color_index = i;
        }
    }

    if (closest_biased_color_distance <= closest_standard_color_distance) {
        shader_results[pixel_index] = closest_biased_color_index;
    }
    else {
        shader_results[pixel_index] = biased_pallet_length + closest_standard_color_index;
    }
}