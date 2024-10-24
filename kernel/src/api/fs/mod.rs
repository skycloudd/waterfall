#[derive(Clone, Copy, Debug)]
pub enum IO {
    Read,
    Write,
}

pub trait FileIO {
    fn read(&mut self, buf: &mut [u8]) -> Option<usize>;
    fn write(&mut self, buf: &[u8]) -> Option<usize>;
    fn close(&mut self);
    fn poll(&mut self, event: IO) -> bool;
}
