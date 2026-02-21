define void @"main" () {
entry:
    %_0 = alloca i32
    store i32 69, ptr %_0
    br label %0
    0:
        %1 = load i32, ptr %_0
        %2 = icmp eq i32 %1, 0
        br i1 %2, label %3, label %4
        3:
            br label %end.0
        %5 = load i32, ptr %_0
        %6 = sub i32 %5, 1
        store i32 %6, ptr %_0
        br label %0
    end.0:
    ret void
}
