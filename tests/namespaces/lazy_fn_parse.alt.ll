define void @"a::lol" () {
entry:
    ret void
}
define void @"a::b::x" () {
entry:
    call void @"a::lol" ()
    ret void
}
define void @"main" () {
entry:
    call void @"a::b::x" ()
    ret void
}
