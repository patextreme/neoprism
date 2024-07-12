use super::contract::hx::HxRpc;

pub mod components;
pub mod explorer;
pub mod resolver;

pub fn escape_html_json(json: &serde_json::Value) -> String {
    let s = serde_json::to_string(&json).unwrap_or_default();
    html_escape::encode_safe(&s).into()
}

pub fn escape_html_rpc(rpc: &HxRpc) -> String {
    let json = serde_json::json!({"rpc": rpc});
    escape_html_json(&json)
}

pub fn html_page(body: String) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html data-theme="retro">

<head>
  <meta charset="utf-8">
  <title>Prism Node UI</title>
  <meta name="author" content="">
  <meta name="description" content="">
  <meta name="viewport" content="width=device-width, initial-scale=1">

  <link href="/assets/tailwind.css" rel="stylesheet">
  <script src="https://unpkg.com/htmx.org@2.0.0" integrity="sha384-wS5l5IKJBvK6sPTKa2WZ1js3d947pvWXbPJ1OmWfEuxLgeHcEbjUUA5i9V5ZkpCw" crossorigin="anonymous"></script>
  <script src="https://cdn.jsdelivr.net/npm/alpinejs@3.14.1/dist/cdn.min.js"></script>
</head>

<body class="font-mono">
  {body}
</body>

</html>
"#
    )
}
