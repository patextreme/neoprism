pub mod form {
    use rocket::FromForm;

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
        GetExplorerDidList { page: Option<u64> },
    }
}

pub mod api {
    use rocket::serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "rocket::serde", rename_all = "camelCase")]
    pub struct DidDocument {
        #[serde(rename(serialize = "@context", deserialize = "@context"))]
        context: DidDocumentContext,
        id: String,
        verificationMethod: Vec<serde_json::Value>,
        authentication: Option<Vec<serde_json::Value>>,
        assertionMethod: Option<Vec<serde_json::Value>>,
        keyAgreement: Option<Vec<serde_json::Value>>,
        capabilityInvocation: Option<Vec<serde_json::Value>>,
        capabilityDelegation: Option<Vec<serde_json::Value>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(crate = "rocket::serde", rename_all = "camelCase", untagged)]
    pub enum DidDocumentContext {
        Str(String),
        List(Vec<String>),
    }
}
