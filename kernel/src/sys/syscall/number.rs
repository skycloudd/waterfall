#[derive(Debug)]
#[repr(usize)]
pub enum Syscall {
    Sleep = 1,
}

impl TryFrom<usize> for Syscall {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Sleep),
            _ => Err(()),
        }
    }
}
