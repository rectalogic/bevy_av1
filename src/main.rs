use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::WindowResolution,
};
use bevy_av1::decoder::decode;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(1280.0, 720.0),
            ..default()
        }),
        ..default()
    }),))
        .add_systems(Startup, setup);

    app.run();
}

/*
 * Image::new does debug_assert_eq! using texture_format.pixel_size() which as implemented won't work for planar formats
 * release mode works around this, but fails: Features Features(TEXTURE_FORMAT_NV12) are required - only on vulkan/dx12
 */
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) -> Result<()> {
    let p = decode()?;
    assert!(matches!(p.pixel_layout(), dav1d::PixelLayout::I420) && p.bit_depth() == 8);
    let width = p.width();
    let height = p.height();
    let mut yuv = Vec::with_capacity(((width * height) + (width * height) / 2) as usize); //XXX is this right?

    // Copy Y plane
    let chunk_length = width as usize;
    for chunk in p
        .plane(dav1d::PlanarImageComponent::Y)
        .chunks_exact(p.stride(dav1d::PlanarImageComponent::Y) as usize)
    {
        yuv.extend_from_slice(&chunk[..chunk_length]);
    }
    // Copy U plane
    let chunk_length = (width / 2) as usize;
    for chunk in p
        .plane(dav1d::PlanarImageComponent::U)
        .chunks_exact(p.stride(dav1d::PlanarImageComponent::U) as usize)
    {
        yuv.extend_from_slice(&chunk[..chunk_length]);
    }
    // Copy V plane
    let chunk_length = (width / 2) as usize;
    for chunk in p
        .plane(dav1d::PlanarImageComponent::V)
        .chunks_exact(p.stride(dav1d::PlanarImageComponent::V) as usize)
    {
        yuv.extend_from_slice(&chunk[..chunk_length]);
    }
    /*
     * SDL_PIXELFORMAT_IYUV
     * https://github.com/libsdl-org/SDL/blob/1c5c3b1479a05196bee38aab101c5d3ef4a8c754/src/render/SDL_yuv_sw.c#L223
    SDL_UpdateYUVTexture(texture, NULL,
        dav1d_pic->data[0], (int)dav1d_pic->stride[0], // Y
        dav1d_pic->data[1], (int)dav1d_pic->stride[1], // U
        dav1d_pic->data[2], (int)dav1d_pic->stride[1]  // V
        );
     */
    let image = Image::new(
        Extent3d {
            width,
            height,
            ..default()
        },
        TextureDimension::D2,
        yuv,
        TextureFormat::NV12,
        RenderAssetUsages::default(),
    );
    let image_handle = images.add(image);
    commands.spawn(Sprite::from_image(image_handle));
    commands.spawn(Camera2d);
    Ok(())
}
