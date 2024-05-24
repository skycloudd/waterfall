#[repr(usize)]
pub enum Syscall {
    Exit = 0,
    Spawn = 1,
    Sleep = 2,
    Uptime = 3,
    Realtime = 4,
    Shutdown = 5,
}

impl TryFrom<usize> for Syscall {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Exit),
            1 => Ok(Self::Spawn),
            2 => Ok(Self::Sleep),
            3 => Ok(Self::Uptime),
            4 => Ok(Self::Realtime),
            5 => Ok(Self::Shutdown),
            _ => Err(()),
        }
    }
}
