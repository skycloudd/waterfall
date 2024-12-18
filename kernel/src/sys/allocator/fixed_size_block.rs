use super::Locked;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;

        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: *mut u8, heap_size: usize) {
        unsafe { self.fallback_allocator.init(heap_start, heap_size) };
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(()) => core::ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                if let Some(node) = allocator.list_heads[index].take() {
                    allocator.list_heads[index] = node.next.take();

                    core::ptr::from_mut::<ListNode>(node).cast::<u8>()
                } else {
                    let block_size = BLOCK_SIZES[index];

                    // only works if all block sizes are powers of two
                    let block_align = block_size;

                    let layout = Layout::from_size_align(block_size, block_align).unwrap();

                    allocator.fallback_alloc(layout)
                }
            }
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            let new_node = ListNode {
                next: allocator.list_heads[index].take(),
            };

            assert!(core::mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(core::mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

            #[allow(clippy::cast_ptr_alignment)]
            // SAFETY: pointer alignment is checked above
            let new_node_ptr = ptr.cast::<ListNode>();

            unsafe { new_node_ptr.write(new_node) };

            allocator.list_heads[index] = Some(unsafe { &mut *new_node_ptr });
        } else {
            let ptr = NonNull::new(ptr).unwrap();

            unsafe { allocator.fallback_allocator.deallocate(ptr, layout) };
        }
    }
}

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());

    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}
