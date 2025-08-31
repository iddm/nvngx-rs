use ash::vk;
use nvngx_sys::v;

// no allocations import needed

pub fn default_subresource_range() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: vk::REMAINING_MIP_LEVELS,
        base_array_layer: 0,
        layer_count: vk::REMAINING_ARRAY_LAYERS,
    }
}

pub fn clear_color_image(
    dev: &ash::Device,
    cb: vk::CommandBuffer,
    image: vk::Image,
    color: [f32; 4],
) {
    let clear = vk::ClearColorValue { float32: color };
    unsafe {
        dev.cmd_clear_color_image(
            cb,
            image,
            vk::ImageLayout::GENERAL,
            &clear,
            std::slice::from_ref(&default_subresource_range()),
        );
    }
}

pub fn copy_buffer_to_image(
    dev: &ash::Device,
    cb: vk::CommandBuffer,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) {
    let region = vk::BufferImageCopy::default()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(
            vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1),
        )
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });
    unsafe {
        dev.cmd_copy_buffer_to_image(
            cb,
            buffer,
            image,
            vk::ImageLayout::GENERAL,
            std::slice::from_ref(&region),
        );
    }
}

pub fn copy_image_to_buffer(
    dev: &ash::Device,
    cb: vk::CommandBuffer,
    image: vk::Image,
    buffer: vk::Buffer,
    width: u32,
    height: u32,
) {
    let region = vk::BufferImageCopy::default()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(
            vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1),
        )
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });
    unsafe {
        dev.cmd_copy_image_to_buffer(
            cb,
            image,
            vk::ImageLayout::GENERAL,
            buffer,
            std::slice::from_ref(&region),
        );
    }
}
