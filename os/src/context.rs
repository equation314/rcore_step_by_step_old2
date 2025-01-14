pub use rcore_exception::TrapFrame;

#[repr(C)]
pub struct Context {
    content_addr: usize // 上下文内容存储的位置
}

impl Context {
    pub unsafe fn null() -> Context {
        Context { content_addr: 0 }
    }

    pub unsafe fn new_kernel_thread(
        entry: extern "C" fn(usize) -> !,
        arg: usize,
        kstack_top: usize,
        satp: usize ) -> Context {
        ContextContent::new_kernel_thread(entry, arg, kstack_top, satp).push_at(kstack_top)
    }

    pub unsafe fn new_user_thread(
        entry: usize,
        ustack_top : usize,
        kstack_top : usize,
        satp : usize
    ) -> Self {
        ContextContent::new_user_thread(entry, ustack_top, satp).push_at(kstack_top)
    }

    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(&mut self, target: &mut Context) {
        asm!(include_str!("process/switch.asm") :::: "volatile");
    }
}

#[repr(C)]
struct ContextContent {
    ra: usize, // 返回地址
    satp: usize, //　二级页表所在位置
    s: [usize; 12], // 被调用者保存的寄存器
    tf: TrapFrame,
}

extern "C" {
    fn trap_return();
}

use core::mem::zeroed;
use riscv::register::sstatus;
impl ContextContent {
    fn new_kernel_thread(entry: extern "C" fn(usize) -> !, arg: usize , kstack_top: usize, satp: usize) -> ContextContent {
        let mut content: ContextContent = unsafe { zeroed() };
        content.ra = entry as usize;
        content.satp = satp;
        content.s[0] = arg;
        let mut _sstatus = sstatus::read();
        _sstatus.set_spp(sstatus::SPP::Supervisor); // 代表 sret 之后的特权级仍为 Ｓ
        content.s[1] = unsafe { core::mem::transmute(_sstatus) };
        content
    }

    fn new_user_thread(entry : usize, ustack_top : usize, satp : usize) -> Self {
        ContextContent {
            ra: trap_return as usize,
            satp,
            s: [0; 12],
            tf: TrapFrame::new(entry, 0, ustack_top).status(sstatus::read()).user().enable_ints(),
        }
    }

    unsafe fn push_at(self, stack_top: usize) -> Context {
        let ptr = (stack_top as *mut ContextContent).sub(1);
        *ptr = self; // 拷贝 ContextContent
        Context { content_addr: ptr as usize }
    }
}
