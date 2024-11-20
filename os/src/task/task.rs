//! Types related to task management & Functions for completely changing TCB

use super::id::TaskUserRes;
use super::{kstack_alloc, KernelStack, ProcessControlBlock, TaskContext};
use crate::trap::TrapContext;
use crate::{mm::PhysPageNum, sync::UPSafeCell};
use alloc::sync::{Arc, Weak};
use core::cell::RefMut;
use alloc::vec::Vec;
use alloc::vec;
/// Task control block structure
pub struct TaskControlBlock {
    /// immutable
    pub process: Weak<ProcessControlBlock>,
    /// Kernel stack corresponding to PID
    pub kstack: KernelStack,
    /// mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// Get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,
    /// It is set when active exit or execution error occurs
    pub exit_code: Option<i32>,
    ///
    pub  mutex_allocation:Vec<usize>, 
    /// 
    pub semaphore_allocation:Vec<usize>,
    /// 
    pub mutex_need:Vec<usize>,
    /// 
    pub semaphore_need:Vec<usize>,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    /// 
    pub fn adjust_mutex_allocation(&mut self, target_id: usize, num: usize, ifadd:bool) {
        if self.mutex_allocation.len() <= target_id {
            // 如果长度不足，追加0直到 target_id 的索引
            self.mutex_allocation.resize(target_id + 1, 0);
        }
        // 增加目标索引的值
        if ifadd{
            self.mutex_allocation[target_id] += num;
        }
        else {
            {
                self.mutex_allocation[target_id] -= num;
            }
        }
        
    }
    /// 
    pub fn adjust_sema_allocation(&mut self, target_id: usize, num: usize, ifadd:bool) {
        if self.semaphore_allocation.len() <= target_id {
            // 如果长度不足，追加0直到 target_id 的索引
            self.semaphore_allocation.resize(target_id + 1, 0);
        }
        // 增加目标索引的值
        if ifadd{
        self.semaphore_allocation[target_id] += num;
        }
        else {
            self.semaphore_allocation[target_id] -= num;
        }
    }
    /// 
    pub fn adjust_mutex_need(&mut self, target_id: usize, num: usize, ifadd:bool) {
        if self.mutex_need.len() <= target_id {
            // 如果长度不足，追加0直到 target_id 的索引
            self.mutex_need.resize(target_id + 1, 0);
        }
        // 增加目标索引的值
        if ifadd{
            self.mutex_need[target_id] += num;
        }
        else {
            self.mutex_need[target_id] -= num;
        }
        
    }
    /// 
    pub fn adjust_sema_need(&mut self, target_id: usize, num: usize, ifadd:bool) {
        if self.semaphore_need.len() <= target_id {
            // 如果长度不足，追加0直到 target_id 的索引
            self.semaphore_need.resize(target_id + 1, 0);
        }
        // 增加目标索引的值
        if ifadd{
            self.semaphore_need[target_id] += num;
        }
        else {
            self.semaphore_need[target_id] -= num;
        }
        
    }

}

impl TaskControlBlock {
    /// Create a new task
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        let process_inner=process.inner_exclusive_access();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,

                    mutex_need:vec![0;process_inner.mutex_list.len()],
                    mutex_allocation:vec![0;process_inner.mutex_list.len()],
                    semaphore_need:vec![0;process_inner.mutex_list.len()],
                    semaphore_allocation:vec![0;process_inner.mutex_list.len()],
                    
                })
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// The execution status of the current process
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// blocked
    Blocked,
}
