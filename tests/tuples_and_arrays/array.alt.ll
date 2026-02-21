define i32 @"main" () {
entry:
    %_0 = alloca [10 x i32]
    %_1 = alloca i32
    %_2 = alloca i32
    %0 = getelementptr inbounds i32, ptr %_0, i64 0
    store i32 1, ptr %0
    %1 = getelementptr inbounds i32, ptr %_0, i64 1
    store i32 2, ptr %1
    %2 = getelementptr inbounds i32, ptr %_0, i64 2
    store i32 3, ptr %2
    %3 = getelementptr inbounds i32, ptr %_0, i64 3
    store i32 4, ptr %3
    %4 = getelementptr inbounds i32, ptr %_0, i64 4
    store i32 5, ptr %4
    %5 = getelementptr inbounds i32, ptr %_0, i64 5
    store i32 6, ptr %5
    %6 = getelementptr inbounds i32, ptr %_0, i64 6
    store i32 7, ptr %6
    %7 = getelementptr inbounds i32, ptr %_0, i64 7
    store i32 8, ptr %7
    %8 = getelementptr inbounds i32, ptr %_0, i64 8
    store i32 9, ptr %8
    %9 = getelementptr inbounds i32, ptr %_0, i64 9
    store i32 10, ptr %9
    store i32 0, ptr %_1
    store i32 9, ptr %_2
    br label %10
    10:
        %11 = load i32, ptr %_1
        %12 = load i32, ptr %_2
        %13 = sext i32 %12 to i64
        %14 = getelementptr inbounds [10 x i32], ptr %_0, i64 0, i64 %13
        %15 = load i32, ptr %14
        %16 = add i32 %11, %15
        store i32 %16, ptr %_1
        %17 = load i32, ptr %_2
        %18 = icmp eq i32 %17, 0
        br i1 %18, label %19, label %20
        19:
            br label %end.10
        %21 = load i32, ptr %_2
        %22 = sub i32 %21, 1
        store i32 %22, ptr %_2
        br label %10
    end.10:
    %23 = load i32, ptr %_1
    ret i32 %23
}
