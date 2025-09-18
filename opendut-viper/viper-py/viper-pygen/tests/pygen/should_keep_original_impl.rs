use viper_pygen::pygen;

struct Fubar;

#[pygen]
impl Fubar {
    fn inc(value: i32) -> i32 {
        value + 1
    }
}

fn main() {
    assert_eq!(Fubar::inc(1), 2);
}
