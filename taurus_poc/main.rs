//This is a generated main file
macro_rules! add_integers {
    ($first:expr, $second:expr) => {
        $first + $second
    };
}

macro_rules! subtract_integers {
    ($first:expr, $second:expr) => {
        $first - $second
    };
}

macro_rules! multiply_integers {
    ($first:expr, $second:expr) => {
        $first * $second
    };
}
fn main() {
    add_integers!(1, 3);
    subtract_integers!(2, 4);
    multiply_integers!(5, 10);
    add_integers!(3, 100);
}
