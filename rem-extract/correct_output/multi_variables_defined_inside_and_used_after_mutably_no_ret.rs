fn foo() {
    let n = 1;
    let (mut k, mut m, o) = fun_name(n);
    k += o;
    m = 1;
}

fn fun_name(n: i32) -> (i32, i32, i32) {
    let mut k = n * n;
    let mut m = k + 2;
    let mut o = m + 3;
    o += 1;
    (k, m, o)
}