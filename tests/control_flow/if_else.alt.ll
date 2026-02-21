define i32 @"cmp_x" (i32, i32) {
entry:
    %2 = alloca i32
    %3 = alloca i32
    store i32 %0, ptr %2
    store i32 %1, ptr %3
    %4 = load i32, ptr %2
    %5 = load i32, ptr %3
    %6 = icmp eq i32 %4, %5
    br i1 %6, label %7, label %8
    7:
        ret i32 69
    8:
        %9 = load i32, ptr %2
        %10 = load i32, ptr %3
        %11 = icmp ugt i32 %9, %10
        br i1 %11, label %12, label %13
        12:
            ret i32 33
        13:
            ret i32 420
}
define i32 @"main" () {
entry:
    %0 = tail call i32 @"cmp_x" (i32 34, i32 35)
    ret i32 %0
}
