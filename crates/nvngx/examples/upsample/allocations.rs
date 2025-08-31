use ash::vk;
use gpu_allocator::vulkan as vkalloc;
use image::{GenericImageView, RgbaImage};

pub struct BufferAllocation {
    pub buffer: vk::Buffer,
    pub allocation: vkalloc::Allocation,
}

pub struct ImageAllocation {
    pub image: vk::Image,
    pub allocation: vkalloc::Allocation,
    pub view: vk::ImageView,
    pub current_stage: vk::PipelineStageFlags2,
    pub current_layout: vk::ImageLayout,
    pub current_access: vk::AccessFlags2,
}

impl ImageAllocation {
    pub fn image_barrier(&mut self, device: &ash::Device, cb: vk::CommandBuffer, new_stage: vk::PipelineStageFlags2, new_access: vk::AccessFlags2, new_layout: vk::ImageLayout) {
        let sub_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS,
        };

        let barrier = vk::ImageMemoryBarrier2::default()
        .image(self.image)
        .subresource_range(sub_range)
        .src_stage_mask(self.current_stage)
        .dst_stage_mask(new_stage)
        .src_access_mask(self.current_access)
        .dst_access_mask(new_access)
        .old_layout(self.current_layout)
        .new_layout(new_layout);

        let dep_info = vk::DependencyInfo::default()
            .image_memory_barriers(std::slice::from_ref(&barrier));
        unsafe {
            device.cmd_pipeline_barrier2(cb, &dep_info);
        }

        self.current_access = new_access;
        self.current_layout = new_layout;
        self.current_stage = new_stage;

    }
}

pub fn load_png_rgba8(path: &str) -> (RgbaImage, u32, u32) {
    let img = image::open(path).expect("failed to open image");
    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8();
    (rgba, w, h)
}

pub fn create_buffer(
    device: &ash::Device,
    allocator: &mut vkalloc::Allocator,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    location: gpu_allocator::MemoryLocation,
) -> BufferAllocation {
    let buffer_ci = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_ci, None) }.expect("create_buffer");

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let allocation = allocator
        .allocate(&vkalloc::AllocationCreateDesc {
            name: "",
            requirements,
            location,
            linear: true,
            allocation_scheme: vkalloc::AllocationScheme::GpuAllocatorManaged,
        })
        .expect("buffer allocation");

    unsafe {
        device
            .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
            .expect("bind buffer memory");
    }

    BufferAllocation { buffer, allocation }
}

// Unified generic image creator; use this for any 2D optimal-tiling image

pub fn create_image_optimal(
    device: &ash::Device,
    allocator: &mut vkalloc::Allocator,
    width: u32,
    height: u32,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
) -> ImageAllocation {
    let image_ci = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let image = unsafe { device.create_image(&image_ci, None) }.expect("create_image");

    let requirements = unsafe { device.get_image_memory_requirements(image) };
    let allocation = allocator
        .allocate(&vkalloc::AllocationCreateDesc {
            name: "",
            requirements,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: vkalloc::AllocationScheme::GpuAllocatorManaged,
        })
        .expect("image allocation");

    unsafe {
        device
            .bind_image_memory(image, allocation.memory(), allocation.offset())
            .expect("bind image memory");
    }

    // Create a default 2D image view for the image
    let subresource_range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: vk::REMAINING_MIP_LEVELS,
        base_array_layer: 0,
        layer_count: vk::REMAINING_ARRAY_LAYERS,
    };
    let view_ci = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(subresource_range);
    let view = unsafe { device.create_image_view(&view_ci, None) }.expect("create image view");

    ImageAllocation { image, allocation, view, current_layout: image_ci.initial_layout, current_access: vk::AccessFlags2::empty(), current_stage: vk::PipelineStageFlags2::empty() }
}

pub fn destroy_buffer(
    device: &ash::Device,
    allocator: &mut vkalloc::Allocator,
    b: BufferAllocation,
) {
    unsafe { device.destroy_buffer(b.buffer, None) };
    allocator
        .free(b.allocation)
        .expect("free buffer allocation");
}

pub fn destroy_image(device: &ash::Device, allocator: &mut vkalloc::Allocator, i: ImageAllocation) {
    unsafe {
        device.destroy_image_view(i.view, None);
        device.destroy_image(i.image, None);
    }
    allocator.free(i.allocation).expect("free image allocation");
}
