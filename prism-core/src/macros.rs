#[macro_export]
macro_rules! location {
    () => {
        crate::utils::Location {
            file: file!(),
            line: line!(),
        }
    };
}
