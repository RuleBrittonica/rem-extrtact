fn foo() -> Option<()> {
    let n = 1;
    let m = fun_name()?;
    let h = 1 + m;
    Some(())
}

fn fun_name() -> Option<i32> {
    let k = foo()?;
    if k == 42 {
        return None;
    }
    let m = k + 1;
    Some(m)
}