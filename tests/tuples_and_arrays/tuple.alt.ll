define i32 @"main" () {
entry:
    %_0 = alloca {i32, i64}
    %_1 = alloca {i64, i32}
    %0 = getelementptr inbounds {i32, i64}, ptr %_0, i64 0, i32 0
    store i32 34, ptr %0
    %1 = getelementptr inbounds {i32, i64}, ptr %_0, i64 0, i32 1
    store i64 400, ptr %1
    %2 = getelementptr inbounds {i64, i32}, ptr %_1, i64 0, i32 0
    store i64 20, ptr %2
    %3 = getelementptr inbounds {i64, i32}, ptr %_1, i64 0, i32 1
    store i32 35, ptr %3
    %4 = getelementptr inbounds {i32, i64}, ptr %_0, i64 0, i32 0
    %5 = load i32, ptr %4
    %6 = getelementptr inbounds {i64, i32}, ptr %_1, i64 0, i32 1
    %7 = load i32, ptr %6
    %8 = add i32 %5, %7
    ret i32 %8
}
