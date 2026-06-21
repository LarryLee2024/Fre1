// Define a macro locally and test it
#[macro_export]
macro_rules! local_test_id {
    ($vis:vis $name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $vis struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}:{}", $prefix, self.0)
            }
        }
    };
}

// Test calling it with crate:: prefix
crate::local_test_id! {
    pub LocalTestId,
    "localtest"
}
