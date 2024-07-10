pub mod form {
    use rocket::FromForm;

    #[derive(Debug, Clone, FromForm)]
    pub struct ResolveDidForm {
        pub did: String,
    }
}
