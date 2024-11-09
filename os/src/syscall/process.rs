//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM, mm::translated_struct_ptr, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, 
        get_sys_call_times, suspend_current_and_run_next, TaskStatus,get_mmap,get_munmap
    }, timer::get_time_us
};


#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let ts=translated_struct_ptr(current_user_token(),_ts);
    *ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    0

}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let ti=translated_struct_ptr(current_user_token(),_ti);         
    *ti = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_sys_call_times(),
        time: get_time_us() /1000,
    };

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _len==0{
        return 0;
    }
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    }
    get_mmap(_start,_len,_port)
    
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    get_munmap(_start,_len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
