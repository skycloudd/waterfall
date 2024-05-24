#[repr(usize)]
pub enum Syscall {
    Exit = 0,
}

impl TryFrom<usize> for Syscall {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Exit),
            _ => Err(()),
        }
    }
}
