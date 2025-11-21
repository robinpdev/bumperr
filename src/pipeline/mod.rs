use std::any::Any;

use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::{prelude::*, render};
use bevy::render::render_resource::*;
use std::any::{TypeId};

pub fn create_render_target(
    // render_device: &RenderDevice,
    render_device: Res<RenderDevice>,
) {
    let tid = render_device.wgpu_device().type_id();
    println!("I AM IN THE FUNCTION");
    println!("{tid:?}");
    // let mut texture = render_device.create_texture(&TextureDescriptor {
    //     label: Some("test"),
    //     size: Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
    //     mip_level_count: 1,
    //     sample_count: 1,
    //     dimension: TextureDimension::D2,
    //     format: TextureFormat::Rgba8UnormSrgb,
    //     usage: TextureUsages::RENDER_ATTACHMENT
    //         | TextureUsages::TEXTURE_BINDING
    //         | TextureUsages::COPY_SRC,
    //     view_formats: todo!(),
        
    //  });

    // return texture
}
