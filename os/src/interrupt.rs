use crate::context::TrapFrame;
use crate::clock::{ TICK, clock_set_next_event };
use rcore_exception::RvHandler;

pub struct TrapHandler;

impl RvHandler for TrapHandler {
    fn handle_timer() {
        clock_set_next_event();
        unsafe{
            TICK = TICK + 1;
            // if TICK % 100 == 0 {
            //     println!("100 ticks!");
            // }
        }
        crate::process::tick();
    }

    fn handle_external() {
        let ch = bbl::sbi::console_getchar() as u8 as char;
        crate::fs::stdio::STDIN.push(ch);
    }

    fn handle_breakpoint(_tf: &mut TrapFrame) {
        panic!("a breakpoint set by kernel");
    }

    fn handle_syscall(tf: &mut TrapFrame) {
        let ret = crate::syscall::syscall(
            tf.x[17],
            [tf.x[10], tf.x[11], tf.x[12]],
            tf,
        );
        tf.sepc += 4;
        tf.x[10] = ret as usize;
    }
}

#[inline(always)]
pub fn enable_and_wfi() {    // 使能中断并等待中断
    unsafe {
        asm!("csrsi sstatus, 1 << 1; wfi" :::: "volatile");
    }
}

#[inline(always)]
pub fn disable_and_store() -> usize {    // 禁用中断并返回当前中断状态
    let sstatus: usize;
    unsafe {
        asm!("csrci sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
    }
    sstatus & (1 << 1)
}

#[inline(always)]
pub fn restore(flags: usize) {    // 根据 flag 设置中断
    unsafe {
        asm!("csrs sstatus, $0" :: "r"(flags) :: "volatile");
    }
}
