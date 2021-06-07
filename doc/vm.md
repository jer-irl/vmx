# Matching Engine Virtual Machine

## Overview

The Virtual Machine (VM) is a turing-complete RISC-like register-based machine with.
The VM is limited by the available "gas," which is consumed by a fixed amount by each particular instruction executed.
Naturally, different instruction opcodes may have different costs of execution.

A program may have associated parameters, which can be updated separately from the program.
These parameters are passed to the program in the global map.

## Registers

All values 2s-complement signed 64-bit integers

- r0-r14
- rp

## Opcodes

- arrins value, arr, idx
- arrget dst, arr, idx
- movimm dst, imm
- mov dst, src
- jmp dst
- jeq/jne/jgt/jge/jlt/jle dst, v0, v1
- add dst, v0, v1
- mul dst, v0, v1
- div dst, v0, v1
- mod dst, v0, v1

## Program parameters

```{}
arr0[0]: param0
arr0[1]: param1
...
arr0[N]: paramN
```

## Book state

TODO

```{}
arr1[0]: min bid price or 0 if none
arr1[price]: #bids at price
arr2[0]: max offer price or 0 if none
arr2[price]: #offers at price
arr3[0]: my min bid price or 0 if none
arr3[price]: #my bids at price
arr4[0]: my max offer price or 0 if none
arr4[price]: #my offers at price
```

## Price revisions

```{}
arr5[0]: 0 if reusing old bids, 1 if erasing old bids
arr5[price]: #bids to add or subtract at price (negative result is error)
arr6[0]: 0 if reusing old offers, 1 if erasing old offers
arr6[price]: #offers to add or subtract at price (negative result is error)
```
