#[unsafe(no_mangle)]
pub extern "C" fn add(left: i32, right: i32) -> i32 {
    let res = left + right;
    println!("{left} + {right} = {res}");
    res
}

#[unsafe(no_mangle)]
pub extern "C" fn sub(left: i32, right: i32) -> i32 {
    let res = left - right;
    println!("{left} - {right} = {res}");
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
        let result = sub(3, 1);
        assert_eq!(result, 2);
    }
}
