# Lab 1: Appetizer

---

## Information

Name: Jia Shengyuan

Email: [jia_shengyuan@stu.pku.edu.cn](jia_shengyuan@stu.pku.edu.cn)

> Please cite any forms of information source that you have consulted during finishing your assignment, except the TacOS documentation, course slides, and course staff.

> With any comments that may help TAs to evaluate your work better, please leave them here

## Booting Tacos

> A1: Put the screenshot of Tacos running example here.

![](image.png)

## Debugging

### First instruction

> B1: What is the first instruction that gets executed?

`auipc   t0,0x0`

> B2: At which physical address is this instruction located?

`0x1000`

### From ZSBL to SBI

> B3: Which address will the ZSBL jump to?

`0x80000000`

### SBI, kernel and argument passing

> B4: What's the value of the argument `hart_id` and `dtb`?

`hart_id = 0, dtb = 0x82200000`

> B5: What's the value of `Domain0 Next Address`, `Domain0 Next Arg1`, `Domain0 Next Mode` and `Boot HART ID` in OpenSBI's output?

`Domain0 Next Address = 0x80200000`
`Domain0 Next Arg1 = 0x82200000`
`Domain0 Next Mode = S-Mode`
`Boot HART ID = 0`

> B6: What's the relationship between the four output values and the two arguments?

`Boot HART ID = hart_id`
`Domain0 Next Arg1 = dtb`

### SBI interfaces

> B7: Inside `console_putchar`, Tacos uses `ecall` instruction to transfer control to SBI. What's the value of register `a6` and `a7` when executing that `ecall`?

`a6 = 0, a7 = 1`

## Kernel Monitor

> C1: Put the screenshot of your kernel monitor running example here. (It should show how your kernel shell respond to `whoami`, `exit`, and `other input`.)

![](run_shell.png)

> C2: Explain how you read and write to the console for the kernel monitor.

Read: I defined a `read_line` function inside the shell branch, which uses `sbi::legacy::console_getchar()` to get input from the console.

Write: call `kprintln!`.