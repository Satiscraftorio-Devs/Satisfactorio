use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, IndexFormat, Queue};

const BUFFER_CAPACITY_MARGIN: f32 = 1.5;
const BUFFER_MIN_CAPACITY: u32 = 4096;

pub struct StandaloneBuffer {
    buffer: Buffer,
    length: u32,
    capacity: u32,
    format: Option<IndexFormat>,
    usage: BufferUsages,
}

impl StandaloneBuffer {
    pub fn from(
        data: &[u8],
        device: &Device,
        queue: &Queue,
        format: Option<IndexFormat>,
        usages: BufferUsages
    ) -> Self{
        let length = data.len() as u32;
        let capacity = BUFFER_MIN_CAPACITY.max((length as f32 * BUFFER_CAPACITY_MARGIN).ceil() as u32);

        // println!("StandaloneBuffer create: length {}, capacity {}", length, capacity);

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(format!("StandaloneBuffer (c: {}, l: {})", capacity, length).as_str()),
            size: capacity as u64 * std::mem::size_of::<u8>() as u64,
            usage: usages,
            mapped_at_creation: false,
        });

        queue.write_buffer(&buffer, 0, data);

        StandaloneBuffer {
            buffer,
            length,
            capacity,
            format: format,
            usage: usages,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        return &self.buffer;
    }

    pub fn length(&self) -> u32 {
        return self.length;
    }

    pub fn capacity(&self) -> u32 {
        return self.capacity;
    }

    pub fn format(&self) -> Option<IndexFormat> {
        return self.format;
    }

    pub fn usages(&self) -> BufferUsages {
        return self.usage;
    }

    pub fn update(&mut self, device: &Device, queue: &Queue, data: &[u8]) {
        let length = data.len() as u32;

        // println!("StandaloneBuffer update: length {}, capacity {}", length, self.capacity);

        if self.capacity >= length {
            self.length = length;
            queue.write_buffer(&self.buffer, 0, data);
        }
        else {
            self.buffer.destroy();
            *self = StandaloneBuffer::from(data, device, queue, self.format, self.usage);
        }
    }
}