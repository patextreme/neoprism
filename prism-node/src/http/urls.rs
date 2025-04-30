use crate::http::ui_explorer::models::PageQuery;
use crate::http::ui_resolver::models::DidQuery;

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
    ($(($ident:ident: $ty:ty)),* $(? $query:ty)?) => {
        #[allow(unused)]
        pub fn url($($ident: $ty),* $(query: $query)?) -> String {
            let base = Self::AXUM.to_string()
                $(.replace(
                        &format!("{{{}}}", stringify!($ident)),
                        &$ident.to_string()
                ))*;

            let full = base $(
                + "?" + &{
                    type _1 = $query; // dummy type to make repitition work
                    serde_qs::to_string(&query).unwrap_or_default()
                }
            )?;

            if full.ends_with('?') {
                full[..full.len() - 1].to_string()
            } else {
                full
            }
        }
    };
    ($(($ident:ident : $ty:ty)),* ,$_:literal  $(, $parts:tt)* $(? $query:ty)?) => {
        expand_make_url!($(($ident: $ty)),* $(, $parts)* $(? $query)?);
    };
    ($_:literal  $(, $parts:tt)* $(? $query:ty)?) => {
        expand_make_url!($($parts),* $(? $query)?);
    }
}

macro_rules! url_def {
    ($ident:ident, $($parts:tt)/ * $(? $query:ty)?) => {
        pub struct $ident;
        impl $ident {
            #[allow(unused)]
            pub const AXUM: &str = expand_axum_url!($($parts),*);

            expand_make_url!($($parts),* $(? $query)?);
        }
    };
}

// assets
url_def!(AssetBase, "assets");
url_def!(AssetStyleSheet, "assets" / "styles.css");

// misc
url_def!(Home, "");
url_def!(Swagger, "swagger-ui");
url_def!(ApiDid, "api" / "dids" / (did: String));
url_def!(Resolver, "resolver" ? Option<DidQuery>);

// explorer
url_def!(Explorer, "explorer" ? Option<PageQuery>);
url_def!(ExplorerDltCursor, "explorer" / "dlt-cursor");
url_def!(ExplorerDidList, "explorer" / "did-list" ? Option<PageQuery>);

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default)]
    pub struct QueryParam {
        page: Option<u32>,
        size: Option<u32>,
        comment: Option<String>,
    }

    url_def!(TestLiveness, "api" / "health" / "liveness");
    url_def!(TestReadiness, "api" / "health" / "readiness");
    url_def!(TestBook, "api" / "books" / (book_id: u32));
    url_def!(TestBookAuthor, "api" / "books" / (book_id: u32) / "author");
    url_def!(TestGhPr, (org: String) / (repo: String) / "pulls" / (pull_id: u32));
    url_def!(TestQuery, "api" / "books" ? Option<QueryParam>);

    #[test]
    fn dynamic_url_axum_url() {
        assert_eq!(TestLiveness::AXUM, "/api/health/liveness");
        assert_eq!(TestLiveness::AXUM, TestLiveness::url());

        assert_eq!(TestReadiness::AXUM, "/api/health/readiness");
        assert_eq!(TestReadiness::AXUM, TestReadiness::url());

        assert_eq!(TestBook::AXUM, "/api/books/{book_id}");
        assert_eq!(TestBook::url(123), "/api/books/123");

        assert_eq!(TestBookAuthor::AXUM, "/api/books/{book_id}/author");
        assert_eq!(TestBookAuthor::url(123), "/api/books/123/author");

        assert_eq!(TestGhPr::AXUM, "/{org}/{repo}/pulls/{pull_id}");
        assert_eq!(
            TestGhPr::url("tokio-rs".to_string(), "axum".to_string(), 123),
            "/tokio-rs/axum/pulls/123"
        );

        assert_eq!(TestQuery::AXUM, "/api/books");
        assert_eq!(TestQuery::url(None), "/api/books");
        assert_eq!(TestQuery::url(Some(Default::default())), "/api/books");
        assert_eq!(
            TestQuery::url(Some(QueryParam {
                page: Some(1),
                ..Default::default()
            })),
            "/api/books?page=1"
        );
        assert_eq!(
            TestQuery::url(Some(QueryParam {
                size: Some(20),
                ..Default::default()
            })),
            "/api/books?size=20"
        );
        assert_eq!(
            TestQuery::url(Some(QueryParam {
                page: Some(1),
                size: Some(20),
                ..Default::default()
            })),
            "/api/books?page=1&size=20"
        );
        assert_eq!(
            TestQuery::url(Some(QueryParam {
                comment: Some("&".to_string()), // must be escaped
                ..Default::default()
            })),
            "/api/books?comment=%26"
        );
    }
}
