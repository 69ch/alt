define void @"foo" (i64, ptr) {
entry:
    %2 = alloca i64
    %3 = alloca ptr
    store i64 %0, ptr %2
    store ptr %1, ptr %3
    br label %4
    4:
        %5 = load i64, ptr %2
        %6 = icmp eq i64 %5, 0
        br i1 %6, label %7, label %8
        7:
            br label %end.4
        %9 = load ptr, ptr %3
        call void %9 (i64* %2)
        br label %4
    end.4:
    ret void
}
define void @".anon.0" (i64*) {
entry:
    %1 = alloca i64*
    store i64* %0, ptr %1
    %2 = load i64*, ptr %1
    %3 = load i64, ptr %2
    %4 = sub i64 %3, 1
    %5 = load i64*, ptr %1
    store i64 %4, ptr %5
    ret void
}
define void @"main" () {
entry:
    call void @"foo" (i64 69, ptr @.anon.0)
    ret void
}
