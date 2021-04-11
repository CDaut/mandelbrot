use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::instance::InstanceExtensions;
use std::io;
use std::io::BufRead;
use vulkano::device::{Device, Features, DeviceExtensions};
use std::sync::Arc;
use vulkano::image::{StorageImage, ImageDimensions};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::format::Format;
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::sync::GpuFuture;
use vulkano::pipeline::ComputePipeline;
use vulkano::buffer::CpuAccessibleBuffer;
use image::{ImageBuffer, Rgba};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::image::view::ImageView;
use vulkano::memory::DedicatedAlloc::Image;

const IMAGE_SIZE: u32 = 1024;

pub fn run() {
    //get a vulcan instance
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("failed to create instance");

    //get all physical devices
    let physical_devices = PhysicalDevice::enumerate(&instance);

    let device: PhysicalDevice;
    let number: usize;

    //let the user select a device or just use the only available one
    if physical_devices.len() != 1 {
        println!("Available GPUs: ");

        for (n, dev) in physical_devices.enumerate() {
            println!("{}: {}", n, dev.name());
        }

        println!("Enter number of GPU to use: ");
        let input: String = io::stdin().lock().lines().next().unwrap().unwrap();
        number = input.parse::<usize>().unwrap();
    } else {
        number = 0;
    }

    //construct a physical device
    device = PhysicalDevice::from_index(&instance, number).unwrap();
    println!("Using GPU \"{}\".", device.name());

    //get the appropriate queue
    let queue_family = device.queue_families()
        .find(|&q| q.supports_compute())
        .expect("couldn't find a queue family supporting graphical and compute");

    //construct the device itself
    let (device, mut queues) = {
        Device::new(device, &Features::none(), &DeviceExtensions::none(),
                    [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };

    //extract the queue
    let queue = queues.next().unwrap();

    //build shader
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            src: "
#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(push_constant) uniform PushConstantData {
  float zoom_factor;
  float re;
  float im;
} pc;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (norm_coordinates - vec2(0.5)) * pc.zoom_factor - vec2(pc.re, pc.im);

    vec2 z = vec2(0.0, 0.0);
    float i;
    for (i = 0.0; i < 1.0; i += 0.004) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }

    vec4 to_write;

    if (i <= 0.3){
        to_write = vec4(0.0, 0.0, i, 1.0);
    }else if(i <= 0.6){
        to_write = vec4(0.0, i, 1.0, 1.0 );
    }else if(i + 0.004 >= 1.0){
        to_write = vec4(0.0, 0.0, 0.0, 1.0 );
    }else{
        to_write = vec4(i, 1.0, 1.0, 1.0 );
    }

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}
"
        }
    }

    let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None)
            .expect("failed to create compute pipeline"),
    );

    let image = StorageImage::new(
        device.clone(),
        ImageDimensions::Dim2d {
            width: IMAGE_SIZE,
            height: IMAGE_SIZE,
            array_layers: 1,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
        .unwrap();

    let layout = compute_pipeline
        .layout()
        .descriptor_set_layout(0)
        .unwrap();

    let set = Arc::new(
        PersistentDescriptorSet::start(
            layout.clone()
        ).add_image(
            ImageView::new(
                image.clone())
                .unwrap())
            .unwrap()
            .build()
            .unwrap());


    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..IMAGE_SIZE * IMAGE_SIZE * 4).map(|_| 0u8),
    )
        .expect("failed to create buffer");

    let push_constants = cs::ty::PushConstantData {
        zoom_factor: 1.0,
        re: 1.0,
        im: 0.0,
    };

    let mut builder = AutoCommandBufferBuilder::new(
        device.clone(), queue.family()).unwrap();

    builder
        .dispatch(
            [IMAGE_SIZE / 8, IMAGE_SIZE / 8, 1],
            compute_pipeline.clone(),
            set.clone(),
            push_constants,
            vec![],
        )
        .unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let buffer_content = buf.read().unwrap();

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
        IMAGE_SIZE,
        IMAGE_SIZE,
        &buffer_content[..]).unwrap();

    image.save("./pictures/image.png").unwrap();
}



