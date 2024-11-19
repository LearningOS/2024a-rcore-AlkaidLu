# 2024-autumn-rCore-ch3

## 编程作业

1. `os/src/task/task.rs`的 `TaskCOntrolBlock` 处加入了一个 `pub sys_call_times: [u32; MAX_SYSCALL_NUM], `项来记录各个系统调用的被调用次数
2. `os/src/task/mod.rs`的 `TaskManager`方法内加入了 `fn increase_sys_call(&self, sys_id:usize)`和 `fn get_sys_call_times(&self)->[u32;MAX_SYSCALL_NUM]`并且加了外部接口
3. `os/src/syscall/mod.rs`的 `syscall`函数内引用了 `increase_sys_call`
4. `os/src/syscall/process.rs`里调用了 `get_sys_call_times`来实现 `sys_task_info`函数

## 简答作业

1. **在** `os`目录下运行命令行 `make run LOG=ERROR`
   **Rustsbi版本为** `RustSBI-QEMU Version 0.2.0-alpha.2`
   **出现报错:**

   ```
    [ERROR] [kernel] .bss [0x8026f000, 0x80298000)
    [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
   ```

   * `ch2b_bad_address.rs`是访问了内核保留的地址，0x0，所以触发了页故障。
   * `ch2b_bad_instructions.rs`是因为 `sret`是一个特权指令，需要在S态，内核态运行，然而这里是在U态，用户态，引发 `IllegalInstruction` 异常。
   * `ch2b_bad_register.rs`程序尝试读取 `sstatus` 寄存器的值。`sstatus` 是一个特权寄存器，只能在 S 态（或更高特权级别）读取 `sstatus`。然而这里是在U态，用户态，引发 `IllegalInstruction` 异常。
2. **函数 **`__alltraps` 和 `__restore` 的作用

   * `__alltraps` 是中断或异常发生时进入的汇编入口函数。它保存用户态的上下文（包括所有寄存器），并将用户栈指针 `sscratch` 和内核栈指针 `sp` 交换。之后，跳转到 `trap_handler` 进行异常处理。
   * `__restore` 在处理完成后负责恢复用户态的上下文。它恢复寄存器值、栈指针，并执行 `sret` 指令，切换回用户态。

   1. `a0`保存了 `trap_handler` 处理后的返回值（用户栈的地址），_restore被用在中断返回和系统调用返回
   2. `t0(sstatus)`：保存并恢复特权级别与中断状态信息
      `t1(spec)`：保存并恢复异常返回地址
      `t2(sscratch)`：保存用户栈指针
   3. `x2(sp)`和 `x4(tp)`被跳过。因为 `sp` 是栈指针，在 `__restore` 函数的前后会被动态修改和交换；`tp` 主要用于线程局部存储（TLS），在用户态应用中一般不涉及，不需要保存和恢复。
   4. `sp`恢复至原始内核栈指针位置，`sscratch`指向用户栈
   5. `sret` 指令（最后一行）完成了状态切换。
      `sret` 从 `sstatus` 恢复特权级别和中断状态，根据 `sstatus` 的值切换至用户态，并使用 `sepc` 中的地址作为返回地址。
   6. `sp`指向内核栈，`sscratch`指向用户栈
   7. **发生在 **`L13: csrrw sp, sscratch, sp`，
   8. **这条指令的作用是将 **`sscratch` 中的用户栈指针和 `sp` 中的内核栈指针进行交换。
      **由于进入 S 态时需要使用内核栈，因此这条指令实现了从用户栈切换到内核栈的过程**

# **荣誉准则**

**警告**

**请把填写了《你的说明》的下述内容拷贝到的到实验报告中。 否则，你的提交将视作无效，本次实验的成绩将按“0”分计。**

1. **在完成本次实验的过程（含此前学习的过程）中，我曾分别与 ****以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
   > *《你交流的对象说明》*
   >
2. **此外，我也参考了 ****以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
   > *《你参考的资料说明》*
   >
3. **我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。**
4. **我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。**
