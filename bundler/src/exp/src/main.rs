pub fn double_number(number_str: &str) -> i32 {
    return number_str.parse::<i32>().unwrap();
}

fn main() {
    let n: i32 = double_number("10");
    assert_eq!(n,20);
}
