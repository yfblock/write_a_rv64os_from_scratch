use alloc::vec::Vec;
use log::info;
use spin::Mutex;

/// FrameAllocator 页帧分配器
/// 知道有哪些页，知道页是否被分配，能分配页

pub struct FrameAllocator {
    start: usize,
    size: usize,
    usage: Vec<bool>
}

impl FrameAllocator {
    /// 创建一个新的页帧分配器
    pub const fn new() -> Self {
        Self {
            start: 0,
            size: 0,
            usage: vec![]
        }
    }

    pub fn add_memory(&mut self, start: usize, size: usize) {
        self.start = start;
        self.size = size;
        self.usage = vec![false; size / 0x1000];
    }

    pub fn alloc(&mut self) -> TrackerFrame {
        for i in 0..self.usage.len() {
            if self.usage[i] == false {
                self.usage[i] = true;
                return TrackerFrame(self.start + i * 0x1000);
            }
        }
        todo!()
    }

    pub fn dealloc(&mut self, addr: usize) {
        let page_index = (addr - self.start) / 0x1000;
        self.usage[page_index] = false;
    }
}

pub struct TrackerFrame(pub usize);

impl Drop for TrackerFrame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self.0);
    }
}

static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::new());

pub fn add_frame_area(start: usize, size: usize) {
    info!("add frame area {:#x} - {:#x} to frame alloctor", start, start + size);
    unsafe {
        core::slice::from_raw_parts_mut(start as *mut u128, size / 16).fill(0);
    }
    FRAME_ALLOCATOR.lock().add_memory(start, size);
    // test frame allocation and test auto drop
    // let mut arr = vec![];
    // for _ in 0..20000 {
    //     let page_start = FRAME_ALLOCATOR.lock().alloc();
    //     info!("frame ptr: {:#x}", page_start.0);
    //     arr.push(page_start);
    // }
}
