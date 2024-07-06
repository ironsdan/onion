use core::result::Result::Ok;
use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage, CopyBufferToImageInfo,
        RecordingCommandBuffer,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo,
        },
        Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions,
    },
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
        StandardMemoryAllocator,
    },
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    DeviceSize, Validated, VulkanError, VulkanLibrary,
};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use super::{
    pipelines::{basic::PSOBasic, texture::PSOTexture},
    render_pass::basic::{RenderPassBasic, RenderPassBasicMSAA},
};

pub struct Pipelines {
    pub basic: PSOBasic,
    pub texture: PSOTexture,
}

pub struct RenderPasses {
    pub basic: RenderPassBasic,
    pub basic_msaa: RenderPassBasicMSAA,
}

pub struct GraphicsContext {
    _instance: Arc<Instance>,
    _debug_callback: DebugUtilsMessenger,
    pub device: Arc<Device>,
    pub window: Arc<Window>,
    pub surface: Arc<Surface>,
    pub gfx_queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub image_index: u32,
    pub final_images: Vec<Arc<Image>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub pipelines: Pipelines,
    pub render_passes: RenderPasses,
    pub memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    pub cb_allocator: Arc<StandardCommandBufferAllocator>,
}

impl GraphicsContext {
    pub fn new<E>(event_loop: &EventLoop<E>) -> Self {
        let library = VulkanLibrary::new().unwrap();

        println!("List of Vulkan debugging layers available to use:");
        let layers = library.layer_properties().unwrap();
        for l in layers {
            println!("\t{}", l.name());
        }

        let layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];

        let _instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_layers: layers,
                enabled_extensions: InstanceExtensions {
                    ext_debug_utils: true,
                    ..Surface::required_extensions(&event_loop).unwrap()
                },
                ..Default::default()
            },
        )
        .expect("failed to create Vulkan instance");

        let _debug_callback = unsafe {
            DebugUtilsMessenger::new(
                _instance.clone(),
                DebugUtilsMessengerCreateInfo {
                    message_severity: DebugUtilsMessageSeverity::ERROR
                        | DebugUtilsMessageSeverity::WARNING
                        | DebugUtilsMessageSeverity::INFO
                        | DebugUtilsMessageSeverity::VERBOSE,
                    message_type: DebugUtilsMessageType::GENERAL
                        | DebugUtilsMessageType::VALIDATION
                        | DebugUtilsMessageType::PERFORMANCE,
                    ..DebugUtilsMessengerCreateInfo::user_callback(
                        DebugUtilsMessengerCallback::new(
                            |message_severity, message_type, callback_data| {
                                let severity = if message_severity
                                    .intersects(DebugUtilsMessageSeverity::ERROR)
                                {
                                    "error"
                                } else if message_severity
                                    .intersects(DebugUtilsMessageSeverity::WARNING)
                                {
                                    "warning"
                                } else if message_severity
                                    .intersects(DebugUtilsMessageSeverity::INFO)
                                {
                                    "information"
                                } else if message_severity
                                    .intersects(DebugUtilsMessageSeverity::VERBOSE)
                                {
                                    "verbose"
                                } else {
                                    panic!("no-impl");
                                };

                                let ty = if message_type.intersects(DebugUtilsMessageType::GENERAL)
                                {
                                    "general"
                                } else if message_type.intersects(DebugUtilsMessageType::VALIDATION)
                                {
                                    "validation"
                                } else if message_type
                                    .intersects(DebugUtilsMessageType::PERFORMANCE)
                                {
                                    "performance"
                                } else {
                                    panic!("no-impl");
                                };

                                println!(
                                    "{} {} {}: {}",
                                    callback_data.message_id_name.unwrap_or("unknown"),
                                    ty,
                                    severity,
                                    callback_data.message
                                );
                            },
                        ),
                    )
                },
            )
            .ok()
        }
        .unwrap();

        let window = Arc::new(
            WindowBuilder::new()
                .with_title("triangle test")
                .with_inner_size(PhysicalSize::new(512.0, 512.0))
                .build(&event_loop)
                .unwrap(),
        );

        let surface = Surface::from_window(_instance.clone(), window.clone()).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = _instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let gfx_queue = queues.next().unwrap();

        let (swapchain, final_images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let image_format = device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0;

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let cb_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo {
                secondary_buffer_count: 32,
                ..Default::default()
            },
        ));

        let ds_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let render_passes = RenderPasses {
            basic: RenderPassBasic::new(gfx_queue.clone(), swapchain.image_format()).unwrap(),
            basic_msaa: RenderPassBasicMSAA::new(gfx_queue.clone(), swapchain.image_format())
                .unwrap(),
        };

        let pipelines = Pipelines {
            basic: PSOBasic::new(
                gfx_queue.clone(),
                render_passes.basic.draw_pass(),
                cb_allocator.clone(),
            ),
            texture: PSOTexture::new(
                gfx_queue.clone(),
                render_passes.basic.draw_pass(),
                cb_allocator.clone(),
                ds_allocator.clone(),
            ),
        };

        Self {
            _instance,
            _debug_callback,
            device,
            window,
            surface,
            gfx_queue,
            swapchain,
            image_index: 0,
            final_images,
            recreate_swapchain: false,
            previous_frame_end,
            render_passes,
            pipelines,
            memory_allocator,
            cb_allocator,
        }
    }

    pub fn start_frame(&mut self) -> Result<Box<dyn GpuFuture>, ()> {
        if self.recreate_swapchain {
            self.recreate_swapchain();
        }

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Err(());
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        self.image_index = image_index;

        let future = self.previous_frame_end.take().unwrap().join(acquire_future);

        Ok(future.boxed())
    }

    pub fn finish_frame(&mut self, after_future: Box<dyn GpuFuture>) {
        let future = after_future
            .then_swapchain_present(
                self.gfx_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                panic!("failed to flush future: {e}");
            }
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let image_extent: [u32; 2] = self.window.inner_size().into();

        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent,
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };

        self.swapchain = new_swapchain;
        self.final_images = new_images;
        self.recreate_swapchain = false;
    }

    pub fn upload_image(&mut self, buf: Subbuffer<[u8]>, extent: [u32; 3]) -> Arc<Image> {
        let mut cb = RecordingCommandBuffer::new(
            self.cb_allocator.clone(),
            self.gfx_queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        let image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        cb.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buf, image.clone()))
            .unwrap();

        self.previous_frame_end = Some(
            cb.end()
                .unwrap()
                .execute(self.gfx_queue.clone())
                .unwrap()
                .boxed(),
        );

        image
    }

    pub fn upload_png(&self, image_bytes: &[u8]) -> (Subbuffer<[u8]>, [u32; 3]) {
        let decoder = png::Decoder::new(image_bytes);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let extent = [info.width, info.height, 1];

        let upload_buffer = Buffer::new_slice(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            (info.width * info.height * 4) as DeviceSize,
        )
        .unwrap();

        reader
            .next_frame(&mut upload_buffer.write().unwrap())
            .unwrap();

        (upload_buffer, extent)
    }
}
