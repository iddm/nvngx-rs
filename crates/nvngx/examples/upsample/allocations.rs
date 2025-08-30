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
    name: &'static str,
) -> BufferAllocation {
    let buffer_ci = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_ci, None) }.expect("create_buffer");

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let allocation = allocator
        .allocate(&vkalloc::AllocationCreateDesc {
            name,
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

pub fn create_image_rgba8_optimal(
    device: &ash::Device,
    allocator: &mut vkalloc::Allocator,
    width: u32,
    height: u32,
    usage: vk::ImageUsageFlags,
    name: &'static str,
) -> ImageAllocation {
    let image_ci = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_UNORM)
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
            name,
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

    ImageAllocation { image, allocation }
}

pub fn destroy_buffer(device: &ash::Device, allocator: &mut vkalloc::Allocator, b: BufferAllocation) {
    unsafe { device.destroy_buffer(b.buffer, None) };
    allocator.free(b.allocation).expect("free buffer allocation");
}

pub fn destroy_image(device: &ash::Device, allocator: &mut vkalloc::Allocator, i: ImageAllocation) {
    unsafe { device.destroy_image(i.image, None) };
    allocator.free(i.allocation).expect("free image allocation");
}