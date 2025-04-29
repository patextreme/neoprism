macro_rules! expand_axum_url {
    () => {
        ""
    };
    (($ident:ident: $_:ty) $(, $parts:tt)*) => {
        concat!("/{", stringify!($ident), "}", expand_axum_url!($($parts),*))
    };
    ($part:literal $(, $parts:tt)*) => {
        concat!("/", $part, expand_axum_url!($($parts),*))
    }
}

macro_rules! expand_make_url {
    ($(($ident:ident: $ty:ty)),*) => {
        #[allow(unused)]
        pub fn new($($ident: $ty),*) -> String {
            Self::AXUM.to_string()
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
            pub const AXUM: &str = expand_axum_url!($($parts),*);

            expand_make_url!($($parts),*);
        }
    };
}

url_def!(AssetBase, "assets");
url_def!(AssetStyleSheet, "assets" / "styles.css");

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
        assert_eq!(TestLiveness::AXUM, "/api/health/liveness");
        assert_eq!(TestLiveness::AXUM, TestLiveness::new());

        assert_eq!(TestReadiness::AXUM, "/api/health/readiness");
        assert_eq!(TestReadiness::AXUM, TestReadiness::new());

        assert_eq!(TestBook::AXUM, "/api/books/{book_id}");
        assert_eq!(TestBook::new(123), "/api/books/123");

        assert_eq!(TestBookAuthor::AXUM, "/api/books/{book_id}/author");
        assert_eq!(TestBookAuthor::new(123), "/api/books/123/author");

        assert_eq!(TestGhPr::AXUM, "/{org}/{repo}/pulls/{pull_id}");
        assert_eq!(
            TestGhPr::new("tokio-rs".to_string(), "axum".to_string(), 123),
            "/tokio-rs/axum/pulls/123"
        );
    }
}
