macro_rules! expand_axum_url {
    () => {
        ""
    };
    (($ident:ident: $_:ty) $(, $parts:tt)*) => {
        format!("/{{{}}}", stringify!($ident)) + &expand_axum_url!($($parts),*)
    };
    ($part:literal $(, $parts:tt)*) => {
        format!("/{}", $part) + &expand_axum_url!($($parts),*)
    }
}

macro_rules! expand_make_url {
    ($(($ident:ident: $ty:ty)),*) => {
            #[allow(unused)]
        pub fn make_url($($ident: $ty),*) -> String {
            Self::axum_url()
                $(.replace(
                        &format!("{{{}}}", stringify!($ident)),
                        &$ident.to_string()
                ))*
        }
    };
    ($(($ident:ident : $ty:ty)),* ,$_:literal  $(, $parts:tt)*) => {
        expand_make_url!($(($ident: $ty)),* $(, $parts)*);
    };
    ($_:literal  $(, $parts:tt)*) => {
        expand_make_url!($($parts),*);
    }
}

macro_rules! url_def {
    ($ident:ident, $($parts:tt)/ *) => {
        pub struct $ident;
        impl $ident {
            #[allow(unused)]
            pub fn axum_url() -> String {
                expand_axum_url!($($parts),*)
            }

            expand_make_url!($($parts),*);
        }
    };
}

url_def!(Home, "");
url_def!(Explorer, "explorer");
url_def!(Resolver, "resolver");
url_def!(Swagger, "swagger-ui");

url_def!(DidResolver, "api" / "dids" / (did: String));

#[cfg(test)]
mod test {
    url_def!(TestLiveness, "api" / "health" / "liveness");
    url_def!(TestReadiness, "api" / "health" / "readiness");
    url_def!(TestBook, "api" / "books" / (book_id: u32));
    url_def!(TestBookAuthor, "api" / "books" / (book_id: u32) / "author");
    url_def!(TestGhPr, (org: String) / (repo: String) / "pulls" / (pull_id: u32));

    #[test]
    fn dynamic_url_axum_url() {
        assert_eq!(TestLiveness::axum_url(), "/api/health/liveness");
        assert_eq!(TestLiveness::axum_url(), TestLiveness::make_url());

        assert_eq!(TestReadiness::axum_url(), "/api/health/readiness");
        assert_eq!(TestReadiness::axum_url(), TestReadiness::make_url());

        assert_eq!(TestBook::axum_url(), "/api/books/{book_id}");
        assert_eq!(TestBook::make_url(123), "/api/books/123");

        assert_eq!(TestBookAuthor::axum_url(), "/api/books/{book_id}/author");
        assert_eq!(TestBookAuthor::make_url(123), "/api/books/123/author");

        assert_eq!(TestGhPr::axum_url(), "/{org}/{repo}/pulls/{pull_id}");
        assert_eq!(
            TestGhPr::make_url("tokio-rs".to_string(), "axum".to_string(), 123),
            "/tokio-rs/axum/pulls/123"
        );
    }
}
