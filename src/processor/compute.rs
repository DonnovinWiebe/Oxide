use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use rayon::iter::IntoParallelIterator;
use wgpu::util::DeviceExt;
use rayon::prelude::*;



const WORKGROUP_SIZE: u32 = 64;
const WORKGROUP_COMPONENT_SIZE: u32 = 8;
const MAX_DISPATCH: u32 = 65535;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuImageInformation {
    width: u32,
    height: u32,
    biased_palette_length: u32,
    standard_palette_length: u32,
}
impl GpuImageInformation {
    fn new(width: u32, height: u32, biased_palette_length: usize, standard_palette_length: usize) -> GpuImageInformation {
        let biased_palette_length = biased_palette_length as u32;
        let standard_palette_length = standard_palette_length as u32;
        GpuImageInformation { width, height, biased_palette_length, standard_palette_length }
    }
}


pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl Gpu {
    pub fn new() -> Self {
        pollster::block_on(async {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .expect("Failed to find GPU adapter");

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .expect("Failed to create GPU device");

            Self { device, queue }
        })
    }

    fn colors_as_vec_u32(colors: &Vec<Rgb<u8>>) -> Vec<u32> {
        colors
            .iter()
            .map(|pixel| { (pixel[0] as u32) | ((pixel[1] as u32) << 8) | ((pixel[2] as u32) << 16) })
            .collect()
    }

    pub fn palettize_evenly(&self, width: u32, height: u32, pixels: &Vec<Rgb<u8>>, palette: &Vec<Rgb<u8>>) -> Vec<Rgb<u8>> {
        // Convert to f32 for GPU
        let u32_pixels: Vec<u32> = Self::colors_as_vec_u32(pixels);
        let u32_palette: Vec<u32> = Self::colors_as_vec_u32(palette);

        // Create GPU buffers
        let dimensions = GpuImageInformation::new(width, height, 0, palette.len());
        let dimensions_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dimensions Buffer"),
            contents: bytemuck::bytes_of(&dimensions),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let pixels_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pixel Buffer"),
            contents: bytemuck::cast_slice(&u32_pixels),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let palette_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Palette Buffer"),
            contents: bytemuck::cast_slice(&u32_palette),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let shader_results_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shader Results Buffer"),
            size: (pixels.len() * size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Load shader
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Palettize Evenly"),
            source: wgpu::ShaderSource::Wgsl(include_str!("palettize_evenly.wgsl").into()),
        });

        // Create pipeline
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: dimensions_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pixels_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: palette_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: shader_results_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        // Execute
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.set_pipeline(&pipeline);

            let dispatch_x = (width + WORKGROUP_COMPONENT_SIZE - 1) / WORKGROUP_COMPONENT_SIZE;
            let dispatch_y = (height + WORKGROUP_COMPONENT_SIZE - 1) / WORKGROUP_COMPONENT_SIZE;
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        // Read results back
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: shader_results_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&shader_results_buffer, 0, &staging_buffer, 0, shader_results_buffer.size());
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let shader_results_data = buffer_slice.get_mapped_range();
        let shader_results: Vec<u32> = bytemuck::cast_slice(&shader_results_data).to_vec();
        drop(shader_results_data);
        staging_buffer.unmap();

        let new_pixels: Vec<Rgb<u8>> = shader_results.iter().map(|&index| {
            palette[index as usize]
        }).collect();
        if new_pixels.len() != pixels.len() { panic!("Shader did not produce correct number of pixels. Expected: {} Produced: {}", pixels.len(), new_pixels.len()); }

        new_pixels
    }

    pub fn palettize_biased(&self, width: u32, height: u32, pixels: &Vec<Rgb<u8>>, biased_palette: &Vec<Rgb<u8>>, standard_palette: &Vec<Rgb<u8>>) -> Vec<Rgb<u8>> {
        // Convert to f32 for GPU
        let u32_pixels: Vec<u32> = Self::colors_as_vec_u32(pixels);
        let u32_biased_palette: Vec<u32> = Self::colors_as_vec_u32(biased_palette);
        let u32_standard_palette: Vec<u32> = Self::colors_as_vec_u32(standard_palette);

        // Create GPU buffers
        let dimensions = GpuImageInformation::new(width, height, biased_palette.len(), standard_palette.len());
        let dimensions_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dimensions Buffer"),
            contents: bytemuck::bytes_of(&dimensions),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let pixels_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pixel Buffer"),
            contents: bytemuck::cast_slice(&u32_pixels),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let biased_palette_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Biased Palette Buffer"),
            contents: bytemuck::cast_slice(&u32_biased_palette),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let standard_palette_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Standard Palette Buffer"),
            contents: bytemuck::cast_slice(&u32_standard_palette),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let shader_results_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shader Results Buffer"),
            size: (pixels.len() * size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Load shader
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Palettize Biased"),
            source: wgpu::ShaderSource::Wgsl(include_str!("palettize_biased.wgsl").into()),
        });

        // Create pipeline
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: dimensions_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pixels_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: biased_palette_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: standard_palette_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: shader_results_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        // Execute
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.set_pipeline(&pipeline);

            let dispatch_x = (width + WORKGROUP_COMPONENT_SIZE - 1) / WORKGROUP_COMPONENT_SIZE;
            let dispatch_y = (height + WORKGROUP_COMPONENT_SIZE - 1) / WORKGROUP_COMPONENT_SIZE;
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        // Read results back
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: shader_results_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&shader_results_buffer, 0, &staging_buffer, 0, shader_results_buffer.size());
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let shader_results_data = buffer_slice.get_mapped_range();
        let shader_results: Vec<u32> = bytemuck::cast_slice(&shader_results_data).to_vec();
        drop(shader_results_data);
        staging_buffer.unmap();

        let new_pixels: Vec<Rgb<u8>> = shader_results.iter().map(|&index| {
            if index >= biased_palette.len() as u32 { standard_palette[index as usize - biased_palette.len()] }
            else { biased_palette[index as usize] }
        }).collect();
        if new_pixels.len() != pixels.len() { panic!("Shader did not produce correct number of pixels. Expected: {} Produced: {}", pixels.len(), new_pixels.len()); }

        new_pixels
    }
}


/// Evenly processes and image using only the colors in a given palette.
pub fn process_evenly(source_image: DynamicImage, palette: Vec<Rgb<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // information
    let (width, height) = source_image.dimensions();
    let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // editing
    let pixels: Vec<Rgb<u8>> = source_image.pixels().map(|pixel| { pixel.2.to_rgb() }).collect();
    let gpu = Gpu::new();
    let new_pixels = gpu.palettize_evenly(width, height, &pixels, &palette);

    // filling the new image with the new pixels
    for x in 0..new_pixels.len() {
        let x_index = (x as u32 % width);
        let y_index = x as u32 / width;
        new_image.put_pixel(x_index, y_index, new_pixels[x]);
    }

    // returns the new image
    new_image
}

/// Processes an image with two palettes with one being preferred.
pub fn process_biased(source_image: DynamicImage, biased_palette: Vec<Rgb<u8>>, standard_palette: Vec<Rgb<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // information
    let (width, height) = source_image.dimensions();
    let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // editing
    let pixels: Vec<Rgb<u8>> = source_image.pixels().map(|pixel| { pixel.2.to_rgb() }).collect();
    let gpu = Gpu::new();
    let new_pixels = gpu.palettize_biased(width, height, &pixels, &biased_palette, &standard_palette);

    // filling the new image with the new pixels
    for x in 0..new_pixels.len() {
        let x_index = (x as u32 % width);
        let y_index = x as u32 / width;
        new_image.put_pixel(x_index, y_index, new_pixels[x]);
    }

    // returns the new image
    new_image
}