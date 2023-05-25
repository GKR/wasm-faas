
fn main() {
    let a = 1;
    
    let b = 3;
    let query_param = std::env::var("query_param").unwrap();
    println!("query_param: {}", query_param);
}
