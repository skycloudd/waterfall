use core::arch::asm;
use number::Syscall;

pub mod number;
mod service;

#[must_use]
pub fn dispatcher(n: usize, arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize) -> usize {
    match n.try_into() {
        Ok(number) => match number {
            Syscall::Sleep => {
                service::sleep(f64::from_bits(arg1 as u64));
                0
            }
        },
        Err(()) => panic!("invalid syscall number {}", n),
    }
}

#[doc(hidden)]
#[must_use]
pub unsafe fn syscall0(n: usize) -> usize {
    let res: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n,
            lateout("rax") res
        );
    }
    res
}

#[doc(hidden)]
#[must_use]
pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let res: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n,
            in("rdi") arg1,
            lateout("rax") res
        );
    }
    res
}

#[doc(hidden)]
#[must_use]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let res: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n,
            in("rdi") arg1, in("rsi") arg2,
            lateout("rax") res
        );
    }
    res
}

#[doc(hidden)]
#[must_use]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let res: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n,
            in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
            lateout("rax") res
        );
    }
    res
}

#[doc(hidden)]
#[must_use]
pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let res: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n,
            in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("r8") arg4,
            lateout("rax") res
        );
    }
    res
}

#[macro_export]
macro_rules! syscall {
    ($n:expr) => {
        $crate::sys::syscall::syscall0($n as usize)
    };
    ($n:expr, $a1:expr) => {
        $crate::sys::syscall::syscall1($n as usize, $a1 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr) => {
        $crate::sys::syscall::syscall2($n as usize, $a1 as usize, $a2 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::sys::syscall::syscall3($n as usize, $a1 as usize, $a2 as usize, $a3 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {
        $crate::sys::syscall::syscall4(
            $n as usize,
            $a1 as usize,
            $a2 as usize,
            $a3 as usize,
            $a4 as usize,
        )
    };
}
