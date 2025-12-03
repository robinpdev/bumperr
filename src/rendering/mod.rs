use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{common::*, pipeline};
use bevy::asset::uuid::Uuid;
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::render::{self, Render};
use bevy::render::render_graph::RenderGraph;
use bevy::{asset::ron::de, prelude::*};

use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};

pub struct ShaderChainPlugin;

#[derive(Component, Default, Clone, ExtractComponent)]
pub struct ShaderChainCamera {
    pub shaders: Vec<String>,
    pub iid: u32,
}


impl Plugin for ShaderChainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // The settings will be a component that lives in the main world but will
            // be extracted to the render world every frame.
            // This makes it possible to control the effect from the main world.
            // This plugin will take care of extracting it automatically.
            // It's important to derive [`ExtractComponent`] on [`PostProcessingSettings`]
            // for this plugin to work correctly.
            ExtractComponentPlugin::<ShaderChainCamera>::default(),
            // The settings will also be the data used in the shader.
            // This plugin will prepare the component for the GPU by creating a uniform buffer
            // and writing the data to that buffer every frame.
            // UniformComponentPlugin::<ShaderChainCamera>::default(),
        ));

        // We need to get the render app from the main app
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // RenderStartup runs once on startup after all plugins are built
        // It is useful to initialize data that will only live in the RenderApp
        render_app
            .add_systems(RenderStartup, init_post_process_pipeline)
            .add_systems(Render, find_chains);

        let world = render_app.world_mut();

        let runner = ViewNodeRunner::new(
            PostProcessNode {
                shader: "abc".to_string(),
            },
            world,
        );

        let Some(mut rendergraph) = world.get_resource_mut::<RenderGraph>() else {
            return;
        };

        rendergraph.sub_graph_mut(Core2d).add_node(
            PostProcessLabel {
                shader: "post_process".to_string(),
            },
            runner,
        );

        render_app
            // Bevy's renderer uses a render graph which is a collection of nodes in a directed acyclic graph.
            // It currently runs on each view/camera and executes each node in the specified order.
            // It will make sure that any node that needs a dependency from another node
            // only runs when that dependency is done.
            //
            // Each node can execute arbitrary work, but it generally runs at least one render pass.
            // A node only has access to the render world, so if you need data from the main world
            // you need to extract it manually or with the plugin like above.
            // Add a [`Node`] to the [`RenderGraph`]
            // The Node needs to impl FromWorld
            //
            // The [`ViewNodeRunner`] is a special [`Node`] that will automatically run the node for each view
            // matching the [`ViewQuery`]
            .add_render_graph_edges(
                Core2d,
                // Specify the node ordering.
                // This will automatically create all required node edges to enforce the given ordering.
                (
                    Node2d::Tonemapping,
                    PostProcessLabel {
                        shader: "post_process".to_string(),
                    },
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }
}

fn find_chains(
    query: Query<(Entity, &mut ShaderChainCamera), Changed<ShaderChainCamera>>,
    mut post_process_pipeline: ResMut<PostProcessPipeline>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
) {
    for (entity, mut chain) in query.iter() {
        if post_process_pipeline.pipelines.contains_key(&chain.iid) {
            continue;
        }else{
            let shader: Handle<Shader> = asset_server.load(&chain.shaders[0]);
            
            
            println!("Initializing post process pipeline");
            // println!("Found shader chain camera with shaders: {:?}", chain.shaders);
            // We need to define the bind group layout used for our pipeline
            let layout = render_device.create_bind_group_layout(
                "post_process_bind_group_layout",
                &BindGroupLayoutEntries::sequential(
                    // The layout entries will only be visible in the fragment stage
                    ShaderStages::FRAGMENT,
                    (
                        // The screen texture
                        texture_2d(TextureSampleType::Float { filterable: true }),
                        // The sampler that will be used to sample the screen texture
                        sampler(SamplerBindingType::Filtering),
                        // The settings uniform that will control the effect
                    ),
                ),
            );
            // We can create the sampler here since it won't change at runtime and doesn't depend on the view

            // Get the shader handle
            // This will setup a fullscreen triangle for the vertex state.
            
            let mut pipeline_ids: Vec<CachedRenderPipelineId> = vec![];
            
            for shader in chain.shaders.iter() {
                let vertex_state = fullscreen_shader.to_vertex_state();
                let shader: Handle<Shader> = asset_server.load(shader);
                let descriptor = RenderPipelineDescriptor {
                        label: Some("post_process_pipeline".into()),
                        layout: vec![layout.clone()],
                        vertex: vertex_state,
                        fragment: Some(FragmentState {
                            shader,
                            // Make sure this matches the entry point of your shader.
                            // It can be anything as long as it matches here and in the shader.
                            targets: vec![Some(ColorTargetState {
                                format: TextureFormat::bevy_default(),
                                blend: None,
                                write_mask: ColorWrites::ALL,
                            })],
                            ..default()
                        }),
                        ..default()
                    };
                let pipeline_id = pipeline_cache
                    // This will add the pipeline to the cache and queue its creation
                    .queue_render_pipeline(descriptor.clone());
                pipeline_ids.push(pipeline_id);
    
            }
            post_process_pipeline.pipelines.insert(chain.iid, pipeline_ids);

        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel {
    pub shader: String,
}
// The post process node used for the render graph
#[derive(Default)]
struct PostProcessNode {
    shader: String,
}

struct PostProcessSpecializer;

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
    pipelines: HashMap<u32, Vec<CachedRenderPipelineId>>,
}

#[derive(Clone, PartialEq, Eq, Hash, SpecializerKey)]
struct CustomPhaseKey {
    shaderhandle: Handle<Shader>,
}

impl Specializer<RenderPipeline> for PostProcessSpecializer {
    type Key = CustomPhaseKey;

    fn specialize(
        &self,
        key: Self::Key,
        pipeline: &mut RenderPipelineDescriptor,
    ) -> Result<Canonical<Self::Key>, BevyError>  {
        pipeline.fragment = Some(FragmentState {
            shader: key.clone().shaderhandle,
            // Make sure this matches the entry point of your shader.
            // It can be anything as long as it matches here and in the shader.
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        });

        return Ok(key);
    }
}

fn init_post_process_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    println!("Initializing post process pipeline");
    // println!("Found shader chain camera with shaders: {:?}", chain.shaders);
    // We need to define the bind group layout used for our pipeline
    let layout = render_device.create_bind_group_layout(
        "post_process_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            // The layout entries will only be visible in the fragment stage
            ShaderStages::FRAGMENT,
            (
                // The screen texture
                texture_2d(TextureSampleType::Float { filterable: true }),
                // The sampler that will be used to sample the screen texture
                sampler(SamplerBindingType::Filtering),
                // The settings uniform that will control the effect
            ),
        ),
    );
    // We can create the sampler here since it won't change at runtime and doesn't depend on the view
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    // Get the shader handle
    let shader: Handle<Shader> = asset_server.load("shaders/post_processing.wgsl");
    // This will setup a fullscreen triangle for the vertex state.
    let vertex_state = fullscreen_shader.to_vertex_state();
    let descriptor = RenderPipelineDescriptor {
            label: Some("post_process_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: vertex_state,
            fragment: Some(FragmentState {
                shader,
                // Make sure this matches the entry point of your shader.
                // It can be anything as long as it matches here and in the shader.
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        };
    let pipeline_id = pipeline_cache
        // This will add the pipeline to the cache and queue its creation
        .queue_render_pipeline(descriptor.clone());

    let hm: HashMap<u32, Vec<CachedRenderPipelineId>> = HashMap::new();

    commands.insert_resource(PostProcessPipeline {
        layout,
        sampler,
        pipeline_id,
        pipelines: hm,
    });
}

// The ViewNode trait is required by the ViewNodeRunner
impl ViewNode for PostProcessNode {
    // The node needs a query to gather data from the ECS in order to do its rendering,
    // but it's not a normal system so we need to define it manually.
    //
    // This query will only run on the view entity
    type ViewQuery = (
        &'static ViewTarget,
        // This makes sure the node only runs on cameras with the PostProcessSettings component
        &'static ShaderChainCamera,
        Entity
        // As there could be multiple post processing components sent to the GPU (one per camera),
        // we need to get the index of the one that is associated with the current view.
    );

    fn update(&mut self, _world: &mut World) {
        
    }

    // Runs the node logic
    // This is where you encode draw commands.
    //
    // This will run on every view on which the graph is running.
    // If you don't want your effect to run on every camera,
    // you'll need to make sure you have a marker component as part of [`ViewQuery`]
    // to identify which camera(s) should run the effect.
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, chain, entity): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // label.0.into
        // Get the pipeline resource that contains the global data we need
        // to create the render pipeline
        let post_process_pipeline = world.resource::<PostProcessPipeline>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame,
        // which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline_ids) = post_process_pipeline.pipelines.get(&chain.iid) else {
            return Ok(());
        };

        for pipeline_id in pipeline_ids.iter() {
            // Get the pipeline from the cache
            let Some(pipeline) = pipeline_cache.get_render_pipeline(*pipeline_id)
            else {
                return Ok(());
            };
    
            // This will start a new "post process write", obtaining two texture
            // views from the view target - a `source` and a `destination`.
            // `source` is the "current" main texture and you _must_ write into
            // `destination` because calling `post_process_write()` on the
            // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
            // texture to the `destination` texture. Failing to do so will cause
            // the current main texture information to be lost.
            let post_process = view_target.post_process_write();
    
            // The bind_group gets created each frame.
            //
            // Normally, you would create a bind_group in the Queue set,
            // but this doesn't work with the post_process_write().
            // The reason it doesn't work is because each post_process_write will alternate the source/destination.
            // The only way to have the correct source/destination for the bind_group
            // is to make sure you get it during the node execution.
            let bind_group = render_context.render_device().create_bind_group(
                "post_process_bind_group",
                &post_process_pipeline.layout,
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                &BindGroupEntries::sequential((
                    // Make sure to use the source view
                    post_process.source,
                    // Use the sampler created for the pipeline
                    &post_process_pipeline.sampler,
                    // Set the settings binding
                    // settings_binding.clone(),
                )),
            );
    
            // Begin the render pass
            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("post_process_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    // We need to specify the post process destination view here
                    // to make sure we write to the appropriate texture.
                    view: post_process.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
    
            // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
            // using the pipeline/bind_group created above
            render_pass.set_render_pipeline(pipeline);
            // By passing in the index of the post process settings on this view, we ensure
            // that in the event that multiple settings were sent to the GPU (as would be the
            // case with multiple cameras), we use the correct one.
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);

        }


        Ok(())
    }
}
