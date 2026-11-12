use std::sync::Arc;

use kurbo::Affine;

use vello_common::paint::{ImageSource, PaintType, TextureRegion};
use vello_common::peniko::ImageAlphaType;
use vello_common::pixmap::{PixelMetadata, Pixmap};
use vello_common::{TextureId, kurbo};
use vello_gpu::{Scene, TextureBindings};
use wgpu::util::DeviceExt;

use crate::render_command::DrawImageCmd;
use crate::resource_mapper::{RendererResourceId, ResourceMapper};
use crate::vello_gpu::render_context::DeviceHandle;
use retgui_resource_manager::ResourceManager;
use retgui_resource_manager::image::ImageResource;
use retgui_resource_manager::resource::Resource;
use retgui_resource_manager::resource_type::ResourceType;

pub(crate) fn upload_image(
    cmd: &DrawImageCmd,
    resource_manager: Arc<ResourceManager>,
    resource_mapper: &mut ResourceMapper,
    texture_bindings: &mut TextureBindings,
    device_handle: &DeviceHandle,
) -> Option<RendererResourceId> {
    let resource = resource_manager.get(&cmd.resource_id)?;
    let image = resource_to_image_resource(resource.as_ref())?;

    // TODO: Handle expired images
    let resource_id = if let Some(resource_id) = resource_mapper.get(&cmd.resource_id) {
        resource_id
    } else {
        let width = u16::try_from(image.get_width()).ok().filter(|&v| v > 0)?;
        let height = u16::try_from(image.get_height()).ok().filter(|&v| v > 0)?;
        let max_dimension = device_handle.device.limits().max_texture_dimension_2d;
        if u32::from(width) > max_dimension || u32::from(height) > max_dimension {
            return None;
        }
        let pixmap = Pixmap::from_parts(
            image.image.as_raw().clone(),
            width,
            height,
            PixelMetadata::new(ImageAlphaType::Alpha, true),
        );
        let image_id = resource_mapper
            .resources
            .values()
            .map(|id| id.0)
            .max()
            .unwrap_or(0)
            .checked_add(1)?;
        let texture = device_handle.device.create_texture_with_data(
            &device_handle.queue,
            &wgpu::TextureDescriptor {
                label: Some("RetGui image"),
                size: wgpu::Extent3d {
                    width: u32::from(width),
                    height: u32::from(height),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            pixmap.data_as_u8_slice(),
        );
        texture_bindings.insert(TextureId(image_id), texture.create_view(&Default::default()));
        let renderer_resource_id = RendererResourceId(image_id);

        resource_mapper.add_mapping(cmd.resource_id.clone(), renderer_resource_id.clone());

        renderer_resource_id
    };

    Some(resource_id)
}

pub(crate) fn draw_image(
    cmd: &DrawImageCmd,
    scene: &mut Scene,
    resource_manager: Arc<ResourceManager>,
    resource_id: RendererResourceId,
) {
    let Some(resource) = resource_manager.get(&cmd.resource_id) else {
        return;
    };
    let Some(image) = resource_to_image_resource(resource.as_ref()) else {
        return;
    };

    let mut transform = Affine::IDENTITY;
    transform = transform.with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64));
    transform = transform.pre_scale_non_uniform(
        cmd.rect.width as f64 / image.get_width() as f64,
        cmd.rect.height as f64 / image.get_height() as f64,
    );
    scene.set_transform(cmd.transform * transform);
    scene.reset_paint_transform();

    let vello_image = vello_common::paint::Image {
        image: ImageSource::external_texture(TextureId(resource_id.0), TextureRegion::Full, true),
        sampler: Default::default(),
    };

    scene.set_paint(PaintType::Image(vello_image));
    scene.fill_rect(&kurbo::Rect::new(
        0.0,
        0.0,
        image.get_width() as f64,
        image.get_height() as f64,
    ));
}

fn resource_to_image_resource(resource: &Resource) -> Option<&ImageResource> {
    if resource.resource_type != ResourceType::Image {
        return None;
    }
    resource.data.downcast_ref::<ImageResource>()
}
