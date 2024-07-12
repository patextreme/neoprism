pub mod form {
    use rocket::FromForm;

    #[derive(Debug, Clone, FromForm)]
    pub struct ResolveDidForm {
        pub did: String,
    }

    #[derive(Debug, Clone, FromForm)]
    pub struct HxRpcForm {
        pub rpc: String,
    }
}

pub mod hx {
    use rocket::serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "rocket::serde")]
    pub enum HxRpc {
        GetExplorerDltCursor {},
        GetExplorerDidList {},
    }
}
