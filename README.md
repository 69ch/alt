# Alt programming language
General purpose compiling programming language in very early stage of development.

## Stack?
LLVM is used mainly. I tried on doing optimizng compiler myself, but soon realized that it will be just completely new project that has nothing to do with this one (I will be making it sometime tho).

## Goal?
General goal is to make simple language that just works, with features inspired by Rust, C, Go and Zig. More control and convenient features, less restrictions and overcompilcations.
From Rust there will be most of syntax and non-restricting features, like convenient algebraic types and functionality and data separation ('impl').
There's no move in direction of asynchronous programming and multi-threading yet, but it's planned to be as convenient as in Go.

## Current plans?
I'm currently very busy with study, so my attention to this project is low, and when I come back time to time I actually don't quite remember what I was doing. But general plans is to:
- Implement data structures, enums, generics
- With structures, implement methods with UFCS
- Implement importing bindings from other files and incremental compilation
- Write tests to it, of course