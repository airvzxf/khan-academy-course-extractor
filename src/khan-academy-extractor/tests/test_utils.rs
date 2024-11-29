#[allow(unused_macros)]
macro_rules! custom_assert_eq {
    (@impl $left:expr, $right:expr, $($msg:expr)?) => {
        let separator_start: &str = &format!("\n{}\nExpected: ", "⥎".repeat(12));
        let separator_middle: &str = &format!("\n{}\n  Actual: ", "⥋".repeat(12));
        let separator_end: &str = &format!("\n{}\n", "⥐".repeat(12));

        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let message = $($msg.map(|m| format!("Message: {}\n", m)).unwrap_or_default())?;
                    panic!("{}{:?}{}{:?}{}{}\n",
                           separator_start, right_val, separator_middle, left_val, separator_end, message);
                }
            }
        }
    };

    ($left:expr, $right:expr) => {
        custom_assert_eq!(@impl $left, $right, None::<&str>)
    };

    ($left:expr, $right:expr, $msg:expr) => {
        custom_assert_eq!(@impl $left, $right, Some($msg))
    };
}

pub(crate) use custom_assert_eq;
