define void @"a::x" () {
entry:
    ret void
}
define void @"main" () {
entry:
    call void @"a::x" ()
    ret void
}
