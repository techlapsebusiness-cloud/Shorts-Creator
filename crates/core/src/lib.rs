//! Project model and timeline maths. The schema arrives in M01.

pub const NAME: &str = "core";

#[cfg(test)]
mod tests {
    #[test]
    fn crate_is_named() {
        assert_eq!(super::NAME, "core");
    }
}
