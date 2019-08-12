use crate::process::scheduler::Scheduler;
use crate::process::structs::*;
use alloc::{ vec::Vec, boxed::Box };

pub struct ThreadInfo {
   pub status: Status,
   pub present: bool,
   thread: Option<Box<Thread>>,
}

pub struct ThreadPool {
    pub threads: Vec<Option<ThreadInfo>>, // 线程信号量的向量
    scheduler: Box<Scheduler>, // 调度算法
}

use crate::process::Tid;

impl ThreadPool {
    pub fn new(size: usize, scheduler: Scheduler) -> ThreadPool {
        ThreadPool {
            threads: {
                let mut th = Vec::new();
                th.resize_with(size, Default::default);
                th
            },
            scheduler: Box::new(scheduler),
        }
    }

    fn alloc_tid(&self) -> Tid {
        for (i, info) in self.threads.iter().enumerate() {
            if info.is_none() {
                return i;
            }
        }
        panic!("alloc tid failed !");
    }

    pub fn add(&mut self, _thread: Box<Thread>) {
        let tid = self.alloc_tid();
        self.threads[tid] = Some(ThreadInfo{
            status: Status::Ready,
            present: true,
            thread: Some(_thread),
        });
        self.scheduler.push(tid);
        println!("tid to alloc: {}", tid);
    }

    pub fn acquire(&mut self) -> Option<(Tid, Box<Thread>)> {
        if let Some(tid) = self.scheduler.pop() {
            let mut thread_info = self.threads[tid].as_mut().expect("thread not exits !");
            thread_info.status = Status::Running(tid);
            return Some((tid, thread_info.thread.take().expect("thread does not exit ")));
        } else {
            return None;
        }
    }

    pub fn retrieve(&mut self, tid: Tid, thread: Box<Thread> ) {
        if (self.threads[tid].is_none()) {
            return;
        }
        let mut thread_info = self.threads[tid].as_mut().expect("thread not exits !");
        if thread_info.present {
            thread_info.thread = Some(thread);
            match thread_info.status {
                Status::Ready | Status::Running(_) => {
                    self.scheduler.push(tid);
                },
                _ => {
                    // println!("do nothing!");
                },
            }
        }
    }

    pub fn tick(&mut self) -> bool {
        // 通知调度算法时钟周期加一，询问是否需要调度
        self.scheduler.tick()
    }

    pub fn exit(&mut self, tid: Tid, code: usize) {
        self.threads[tid] = None;
        self.scheduler.exit(tid);
        println!("exit code: {}", code);
    }

    pub fn wakeup(&mut self, tid: Tid) {
        let proc = self.threads[tid].as_mut().expect("thread not exist");
        if proc.present {
            proc.status = Status::Ready;
            self.scheduler.push(tid);
        } else {
            panic!("try to sleep an null thread !");
        }
    }
}