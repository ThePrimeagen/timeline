
pub fn test() {
    let mut x = 5;
    {
        let y = 10;
        {
            x = y;
        }
    }

    println!("{}", x);
}

