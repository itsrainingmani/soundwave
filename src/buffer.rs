use std::borrow::{Borrow, BorrowMut};

pub struct BufferWrapper<T>(pub Vec<T>);

impl Borrow<[u8]> for BufferWrapper<u32> {
    fn borrow(&self) -> &[u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const u8, self.0.len() * 4) }
    }
}
impl BorrowMut<[u8]> for BufferWrapper<u32> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        // Safe for alignment: align_of(u8) <= align_of(u32)
        // Safe for cast: u32 can be thought of as being transparent over [u8; 4]
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut u8, self.0.len() * 4) }
    }
}
impl Borrow<[u32]> for BufferWrapper<u32> {
    fn borrow(&self) -> &[u32] {
        self.0.as_slice()
    }
}
impl BorrowMut<[u32]> for BufferWrapper<u32> {
    fn borrow_mut(&mut self) -> &mut [u32] {
        self.0.as_mut_slice()
    }
}

impl Borrow<[u32]> for BufferWrapper<u8> {
    // reverse the borrow for u32 -> u8 i guess?
    fn borrow(&self) -> &[u32] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const u32, self.0.len() / 4) }
    }
}

impl BorrowMut<[u32]> for BufferWrapper<u8> {
    fn borrow_mut(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.0.as_mut_ptr() as *mut u32, self.0.len() / 4) }
    }
}

impl Borrow<[u8]> for BufferWrapper<u8> {
    fn borrow(&self) -> &[u8] {
        self.0.as_slice()
    }
}
impl BorrowMut<[u8]> for BufferWrapper<u8> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }
}
