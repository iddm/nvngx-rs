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

fn barrier_image(
    dev: &ash::Device,
    cb: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_stage: vk::PipelineStageFlags,
    dst_stage: vk::PipelineStageFlags,
    src_access: vk::AccessFlags,
    dst_access: vk::AccessFlags,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(default_subresource_range())
        .src_access_mask(src_access)
        .dst_access_mask(dst_access);
    unsafe {
        dev.cmd_pipeline_barrier(
            cb,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier),
        );
    }
}

pub fn to_transfer_dst(dev: &ash::Device, cb: vk::CommandBuffer, image: vk::Image) {
    barrier_image(
        dev,
        cb,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::AccessFlags::empty(),
        vk::AccessFlags::TRANSFER_WRITE,
    );
}

pub fn to_shader_read(dev: &ash::Device, cb: vk::CommandBuffer, image: vk::Image) {
    barrier_image(
        dev,
        cb,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::COMPUTE_SHADER,
        vk::AccessFlags::TRANSFER_WRITE,
        vk::AccessFlags::SHADER_READ,
    );
}

pub fn output_to_general(dev: &ash::Device, cb: vk::CommandBuffer, image: vk::Image) {
    barrier_image(
        dev,
        cb,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::GENERAL,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::COMPUTE_SHADER,
        vk::AccessFlags::empty(),
        vk::AccessFlags::SHADER_WRITE,
    );
}

pub fn output_to_transfer_src(dev: &ash::Device, cb: vk::CommandBuffer, image: vk::Image) {
    barrier_image(
        dev,
        cb,
        image,
        vk::ImageLayout::GENERAL,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        vk::PipelineStageFlags::COMPUTE_SHADER,
        vk::PipelineStageFlags::TRANSFER,
        vk::AccessFlags::SHADER_WRITE,
        vk::AccessFlags::TRANSFER_READ,
    );
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
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
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
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
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
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            buffer,
            std::slice::from_ref(&region),
        );
    }
}

// No destroy wrapper needed; use allocations::destroy_image directly.
