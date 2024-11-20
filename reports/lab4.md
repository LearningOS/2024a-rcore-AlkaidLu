# 代码功能

# 问答题

## ch6


 Inode是文件系统的起点 ：

* 根 inode 对应文件系统的根目录 `/`，它存储了指向所有顶层目录和文件的元数据和数据块地址。
* 在操作系统挂载文件系统时，根 inode 是第一个被加载和验证的 inode。它的元数据决定了文件系统是否完整和可用。

在操作系统挂载文件系统时，根 inode 是第一个被加载和验证的 inode。它的元数据决定了文件系统是否完整和可用。

## ch7

`pipe` 是 UNIX/Linux 系统中实现进程间通信的一种重要机制，常用于父子进程或具有亲缘关系的进程之间传递数据。一个经典的实际应用是 Linux 终端中的  **管道操作符 (`|`)** ，它允许将一个程序的输出作为另一个程序的输入。


```cat
cat file.txt | wc -l

```

Linux 会创建一个 `pipe`，将 `cat file.txt` 的输出通过管道传递给 `wc -l`。


多进程通信可以用共享内存。

# **荣誉准则**

**警告**

**请把填写了《你的说明》的下述内容拷贝到的到实验报告中。 否则，你的提交将视作无效，本次实验的成绩将按“0”分计。**

1. **在完成本次实验的过程（含此前学习的过程）中，我曾分别与 ****以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > *《你交流的对象说明》*

2. **此外，我也参考了 ****以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > *《你参考的资料说明》*

3. **我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。**

4. **我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。**