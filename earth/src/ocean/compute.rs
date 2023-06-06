//! This module implements a compute shader plugin.
//! The code is based
//! heavily on the [bevy compute shader
//! example](https://github.com/bevyengine/bevy/blob/e954b8573c085a01c62007c4c6232870e0b5c891/examples/shader/compute_shader_game_of_life.rs).
//!
//! The compute shader pulls in a noisy red/green seed texture and
//! outputs thier sum in a "displacement" texture's blue channel. The
//! seed is also copied to the red and green channels of a "normal"
//! texture.

// Additional references:
// - The bevy documentation
// - [The WebGPU official docs](https://gpuweb.github.io/gpuweb/).
// - The first paragraph of the "Texture Views and Samplers" section
//   of [this tutorial](https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/),
//   accessed Wed 14 Dec 2022"

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        RenderApp,
        render_asset::RenderAssets,
        RenderSet,
        renderer::{
            RenderContext,
            RenderDevice,
            RenderQueue,
        },
        render_graph::{
            Node as RenderGraphNode,
            NodeRunError,
            RenderGraph,
            RenderGraphContext,
        },
        render_resource::*,
        texture::ImageSampler,
    },
};

use crate::rng::EarthRng;

use image::EncodableLayout;

use std::borrow::Cow;

pub const TEXTURE_SIZE: UVec2 = UVec2{ x: 256, y: 256 };
pub const WORKGROUP_DIMENSION: u32 = 8;
pub const RENDER_NODE_NAME: &str = "ocean_compute_node";

/// The plugin that does all setup for the compute pipeline
pub struct OceanComputePlugin;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

/// This holds all three images used by the ocean compute shader
///
/// This is analogous to `GameOfLifeImage` from the [bevy compute
/// example](https://github.com/bevyengine/bevy/blob/v0.10.1/examples/shader/compute_shader_game_of_life.rs).
#[derive(Clone, Debug, Resource, ExtractResource)]
pub struct OceanComputeImages {
    pub normal: Handle<Image>,
    pub displacement: Handle<Image>,
    pub seed: Handle<Image>,
}

// Implemented according to
// https://en.wikipedia.org/wiki/Box%E2%80%93Muller_transform
fn box_muller(rng: &fastrand::Rng) -> [f32; 2] {
    let [a, b] = [rng.f32(), rng.f32()];
    let phi = b * std::f32::consts::TAU;
    let r = (-2.0f32 * a.ln()).sqrt();

    [r * phi.cos(), r * phi.sin()]
}

fn create_seed(rng: &fastrand::Rng) -> Image {
    let texture_extent = Extent3d {
        width: TEXTURE_SIZE.x,
        height: TEXTURE_SIZE.y,
        depth_or_array_layers: 1
    };

    let mut data = Vec::new();
    let size = TEXTURE_SIZE.x * TEXTURE_SIZE.y;
    
    for _ in 0..size {
        data.extend_from_slice(box_muller(rng).as_bytes());
    }

    // We only need two channels for the seed.
    Image::new(
        texture_extent,
        TextureDimension::D2,
        data,
        TextureFormat::Rg32Float,
    )
}

fn create_textures(mut commands: Commands, rng: Res<EarthRng>, mut images: ResMut<Assets<Image>>) {
    let texture_extent = Extent3d {
        width: TEXTURE_SIZE.x,
        height: TEXTURE_SIZE.y,
        depth_or_array_layers: 1
    };

    let black: [u8; 4] = [0, 0, 0, 0xff];

    let mut seed = create_seed(&rng.0.lock().unwrap());

    let mut normal = Image::new_fill(
        texture_extent,
        TextureDimension::D2,
        &black,
        TEXTURE_FORMAT,
    );

    let mut displacement = Image::new_fill(
        texture_extent,
        TextureDimension::D2,
        &black,
        TEXTURE_FORMAT,
    );

    // Read from the seed, write to the normal and displacement.
    seed.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;

    normal.texture_descriptor.usage = TextureUsages::COPY_DST
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::STORAGE_BINDING;

    displacement.texture_descriptor.usage = TextureUsages::COPY_DST
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::STORAGE_BINDING;

    let mut sampler_descriptor = ImageSampler::linear_descriptor();
    sampler_descriptor.address_mode_u = AddressMode::Repeat;
    sampler_descriptor.address_mode_v = AddressMode::Repeat;

    normal.sampler_descriptor = ImageSampler::Descriptor(sampler_descriptor.clone());
    displacement.sampler_descriptor = ImageSampler::Descriptor(sampler_descriptor);

    let seed_handle = images.add(seed);
    let normal_handle = images.add(normal);
    let displacement_handle = images.add(displacement);

    commands.insert_resource(OceanComputeImages {
        seed: seed_handle,
        normal: normal_handle,
        displacement: displacement_handle,
    });
}

impl Plugin for OceanComputePlugin {
    fn build(&self, app: &mut App) {
        // Ensure that the textures are created prior to sprites.
        app.add_startup_system(create_textures.in_base_set(StartupSet::PreStartup))
            .add_plugin(ExtractResourcePlugin::<OceanComputeImages>::default())
            .add_plugin(ExtractResourcePlugin::<Wind>::default())
            .world
            .get_resource_or_insert_with(Wind::default);

        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<OceanComputePipeline>()
            .add_system(prepare_ocean.in_set(RenderSet::Prepare))
            .add_system(create_bind_groups.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>()
            .expect("no render graph for compute pass");

        render_graph.add_node::<OceanRenderNode>(RENDER_NODE_NAME, OceanRenderNode::Loading);
        render_graph.add_node_edge(RENDER_NODE_NAME, bevy::render::main_graph::node::CAMERA_DRIVER)
    }
}

#[derive(ShaderType)]
struct OceanParameters {
    wind: Vec2,
    time: f32,
}

#[derive(Resource)]
struct GpuOcean {
    parameters: UniformBuffer<OceanParameters>,
}

fn prepare_ocean(
    mut commands: Commands,
    wind: Res<Wind>,
    time: Res<Time>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let mut parameters = UniformBuffer::from(OceanParameters {
        time: time.elapsed_seconds(),
        wind: wind.0,
    });

    parameters.write_buffer(&device, &queue);
    
    commands.insert_resource(GpuOcean {
        parameters,
    });
}

fn create_bind_groups(
    mut commands: Commands,
    gpu_ocean: Res<GpuOcean>,
    ocean_images: Res<OceanComputeImages>,
    pipeline: Res<OceanComputePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
) {
    let seed_view = &gpu_images[&ocean_images.seed].texture_view;
    let displacement_view = &gpu_images[&ocean_images.displacement].texture_view;
    let normal_view = &gpu_images[&ocean_images.normal].texture_view;

    let entries = [
        BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(seed_view)
        },
        BindGroupEntry {
            binding: 1,
            resource: BindingResource::TextureView(displacement_view),
        },
        BindGroupEntry {
            binding: 2,
            resource: BindingResource::TextureView(normal_view),
        },
        BindGroupEntry {
            binding: 3,
            resource: gpu_ocean.parameters.binding().expect("No binding for ocean parameters buffer"),
        },
    ];

    let bind_group_descriptor = BindGroupDescriptor {
        label: Some("Ocean Compute Bind Group"),
        layout: &pipeline.bind_group_layout,
        entries: &entries,
    };
    
    let bind_group = render_device.create_bind_group(&bind_group_descriptor);

    commands.insert_resource(OceanComputeBindGroup(bind_group));
}

/// The wind resource holds a simple, global vector representing its direction
#[derive(Clone, Copy, Debug, Resource, ExtractResource)]
pub struct Wind(Vec2);

impl Default for Wind {
    fn default() -> Self {
        Self(Vec2::from_angle(std::f32::consts::FRAC_PI_4))
    }
}

// Analogous to GameOfLifeImageBindGroup
#[derive(Clone, Debug, Resource)]
struct OceanComputeBindGroup(BindGroup);

// Analogous to GameOfLifeComputePipeline from the example
#[derive(Clone, Debug, Resource)]
struct OceanComputePipeline {
    bind_group_layout: BindGroupLayout,
    pipeline_id: CachedComputePipelineId,
}

fn get_cached_pipeline_state(world: &World) -> &CachedPipelineState {
    let pipeline = world.get_resource::<OceanComputePipeline>()
        .expect("no ocean compute pipeline to check the state of");
        
    let pipeline_cache = world.get_resource::<PipelineCache>()
        .expect("no pipeline cache during state check");

    pipeline_cache.get_compute_pipeline_state(pipeline.pipeline_id)
}

fn get_cached_pipeline(world: &World) -> &ComputePipeline {
    let pipeline = world.get_resource::<OceanComputePipeline>()
        .expect("no ocean compute pipeline from which to get cache id");

    let pipeline_cache = world.get_resource::<PipelineCache>()
        .expect("no pipeline cache from which to get pipeline");

    pipeline_cache.get_compute_pipeline(pipeline.pipeline_id)
        .expect("no ocean compute pipeline cached under the id")
}

impl OceanComputePipeline {
    fn create_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        let layout_entries = [
            // The seed texture binding
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Displacement storage texture
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    view_dimension: TextureViewDimension::D2,
                    format: TEXTURE_FORMAT,
                },
                count: None,
            },
            // Normal storage texture
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    view_dimension: TextureViewDimension::D2,
                    format: TEXTURE_FORMAT,
                },
                count: None,
            },
            // Uniform buffer for wind and time
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(OceanParameters::min_size())
                },
                count: None,
            }
        ];

        let layout_descriptor = BindGroupLayoutDescriptor {
            label: Some("Ocean Compute Bind Group Layout"),
            entries: &layout_entries,
        };

        render_device.create_bind_group_layout(&layout_descriptor)
    }

    fn enqueue_pipeline(
        pipeline_cache: &PipelineCache,
        bind_group_layout: BindGroupLayout,
        shader: Handle<Shader>,
    ) -> CachedComputePipelineId {
        let pipeline_descriptor = ComputePipelineDescriptor {
            label: Some(Cow::from("Ocean Compute Pipeline")),
            push_constant_ranges: Vec::new(),
            layout: vec![bind_group_layout],
            shader,
            shader_defs: Vec::new(),
            entry_point: Cow::from("compute"),
        };

        pipeline_cache.queue_compute_pipeline(pipeline_descriptor)
    }
}

impl FromWorld for OceanComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>()
            .expect("no render device on which to create compute pipeline");

        let bind_group_layout = Self::create_bind_group_layout(render_device);

        let shader = world.get_resource::<AssetServer>()
            .expect("cannot load compute shader: no asset server for render world")
            .load("shaders/ocean_compute.wgsl");

        let pipeline_cache = world.get_resource::<PipelineCache>()
            .expect("no pipeline cache on which to enqueue compute pipeline");

        let pipeline_id = Self::enqueue_pipeline(pipeline_cache, bind_group_layout.clone(), shader);

        OceanComputePipeline {
            bind_group_layout,
            pipeline_id,
        }
    }
}

enum OceanRenderNode {
    Loading,
    Ready,
}

impl OceanRenderNode {
    fn is_pipeline_ready(&mut self, world: &mut World) -> bool {
        let pipeline_state = get_cached_pipeline_state(world);
            
        match pipeline_state {
            CachedPipelineState::Ok(_) => true,
            CachedPipelineState::Queued => false,
            CachedPipelineState::Err(err) =>
                panic!("error during ocean pipeline creation: {err}"),
        }
    }

    fn is_loading(&self) -> bool {
        matches!(self, OceanRenderNode::Loading)
    }

    fn mark_ready(&mut self) {
        *self = Self::Ready;
    }
}

impl RenderGraphNode for OceanRenderNode {
    fn update(&mut self, world: &mut World) {
        if self.is_loading() && self.is_pipeline_ready(world) {
            self.mark_ready()
        }
    }

    fn run(
        &self,
        _graph_context: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World
    ) -> Result<(), NodeRunError> {
        if self.is_loading() {
            return Ok(())
        }

        let compute_pass_descriptor = ComputePassDescriptor {
            label: Some("Ocean Compute Pass"),
        };

        // Any way we could cache these commands in the node?
        let mut compute_pass = render_context
            .command_encoder()
            .begin_compute_pass(&compute_pass_descriptor);

        let bind_group = world.get_resource::<OceanComputeBindGroup>()
            .expect("Could not get ocean compute bind group during command encoding.");

        compute_pass.set_bind_group(0, &bind_group.0, &[]);
        compute_pass.set_pipeline(get_cached_pipeline(world));
        compute_pass.dispatch_workgroups(WORKGROUP_DIMENSION, WORKGROUP_DIMENSION, 1);

        Ok(())
    }
}
