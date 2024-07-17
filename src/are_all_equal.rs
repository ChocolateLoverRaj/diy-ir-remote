// https://sts10.github.io/2019/06/06/is-all-equal-function.html
// https://mastodon.technology/@bugaevc/102226891784062955
pub trait AreAllEqual {
    fn are_all_equal(self) -> bool;
}

impl<T: Eq> AreAllEqual for &[T] {
    fn are_all_equal(self) -> bool {
        self.windows(2).all(|w| w[0] == w[1])
    }
}
