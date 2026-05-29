#[derive(Debug)]
pub struct RenderError(String);

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Render error: {}", self.0)
    }
}

impl std::error::Error for RenderError {}

pub type Result<T> = std::result::Result<T, RenderError>;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point {
    pub lat: f32,
    pub lon: f32,
    pub alt: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewportTransform {
    pub center_lat: f32,
    pub center_lon: f32,
    pub zoom: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub lod_threshold: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OutputPoint {
    pub screen_x: f32,
    pub screen_y: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub visible: u32,
}

const SHADER_SOURCE: &str = include_str!("../../ui-control-center/src/shaders/spatial_transform.wgsl");
const MAX_POINTS: u64 = 1_000_000;

pub struct RenderPipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    input_buffer: wgpu::Buffer,
    viewport_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl RenderPipeline {
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| RenderError("No suitable GPU adapter found".into()))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Render Engine"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| RenderError(format!("Failed to create device: {}", e)))?;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Spatial Transform Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Input Points"),
            size: (MAX_POINTS as usize * std::mem::size_of::<Point>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Viewport Transform"),
            size: std::mem::size_of::<ViewportTransform>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Points"),
            size: (MAX_POINTS as usize * std::mem::size_of::<OutputPoint>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (MAX_POINTS as usize * std::mem::size_of::<OutputPoint>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: viewport_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Spatial Transform Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            compute_pipeline,
            input_buffer,
            viewport_buffer,
            output_buffer,
            staging_buffer,
            bind_group,
            width: 1920,
            height: 1080,
        })
    }

    pub fn update_buffers(&mut self, data: &[u8]) -> Result<()> {
        let points: &[Point] = bytemuck::cast_slice(data);
        let slice_size = (points.len() * std::mem::size_of::<Point>()) as u64;
        let slice_size = slice_size.min(self.input_buffer.size());

        self.queue.write_buffer(&self.input_buffer, 0, &data[..slice_size as usize]);

        let viewport = ViewportTransform {
            center_lat: 37.7749,
            center_lon: -122.4194,
            zoom: 12.0,
            screen_width: self.width as f32,
            screen_height: self.height as f32,
            lod_threshold: 3.0,
        };

        self.queue.write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::bytes_of(&viewport),
        );

        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let point_count = (self.input_buffer.size() / std::mem::size_of::<Point>() as u64)
            .min(MAX_POINTS);

        let workgroup_count = ((point_count + 255) / 256) as u32;

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            },
        );

        {
            let mut compute_pass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor {
                    label: Some("Spatial Transform Pass"),
                    timestamp_writes: None,
                },
            );

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        encoder.copy_buffer_to_buffer(
            &self.output_buffer,
            0,
            &self.staging_buffer,
            0,
            self.output_buffer.size(),
        );

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}
