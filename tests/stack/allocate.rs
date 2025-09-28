use core::alloc::Layout;

use wait_free_arena::{ArenaAllocatorImpl, StackAllocator};

#[test]
fn alloc_basic() {
    let arena: StackAllocator<10> = StackAllocator::new();
    let one = arena.bump_alloc(Layout::new::<u16>()).unwrap();
    unsafe { one.as_mut_ptr().write(42) };
    let two = arena.bump_alloc(Layout::new::<u64>()).unwrap();
    unsafe { two.as_mut_ptr().write(42) };
    assert!(arena.bump_alloc(Layout::new::<u8>()).is_err())
}
