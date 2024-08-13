#[macro_export]
macro_rules! location {
    () => {
        crate::util::Location {
            file: file!(),
            line: line!(),
        }
    };
}
