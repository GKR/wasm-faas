
fn main() {
    let a = 1;

    let b = 3;

    check(a, b);

    let query_param = std::env::var("query_param").unwrap();
    println!("query_param: {}", query_param);
}

fn check(a: i32, b: i32) {
    println!("check({}, {})", a, b);
}