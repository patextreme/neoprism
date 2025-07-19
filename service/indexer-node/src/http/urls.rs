use lazybe::macros::typed_uri;

use crate::http::ui_explorer::models::PageQuery;
use crate::http::ui_resolver::models::DidQuery;

// assets
typed_uri!(AssetBase, "assets");
typed_uri!(AssetStyleSheet, "assets" / "styles.css");

// misc
typed_uri!(Home, "");
typed_uri!(Swagger, "swagger-ui");

// api
typed_uri!(ApiDid, "api" / "dids" / (did: String));
typed_uri!(ApiHealth, "api" / "_system" / "health");
typed_uri!(ApiBuildMeta, "api" / "_system" / "build");

// resolver
typed_uri!(Resolver, "resolver" ? Option<DidQuery>);

// explorer
typed_uri!(Explorer, "explorer" ? Option<PageQuery>);
typed_uri!(ExplorerDltCursor, "explorer" / "dlt-cursor");
typed_uri!(ExplorerDidList, "explorer" / "did-list" ? Option<PageQuery>);
