pub fn f() {
    loop {
        if let ControlFlow::Break(_) = fun_name() {
            continue;
        }

        if false {
            break;
        }
    }
}

fn fun_name() -> ControlFlow<()> {
    if true {
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}