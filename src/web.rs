use std::{cmp::Ordering, fs};

use axum::{Router, extract::Path, response::Html, routing::get};

use crate::{Config, Storage, mermaid::Mermaid};

pub const HEART: &str = "<svg style=\"transform: translateY(3px); height: 1em;\" class=\"heart\" viewBox=\"0 0 16 16\" fill=\"#a6e3a1\" xmlns=\"http://www.w3.org/2000/svg\"><g id=\"SVGRepo_bgCarrier\" stroke-width=\"0\"></g><g id=\"SVGRepo_tracerCarrier\" stroke-linecap=\"round\" stroke-linejoin=\"round\"></g><g id=\"SVGRepo_iconCarrier\"> <path d=\"M1.24264 8.24264L8 15L14.7574 8.24264C15.553 7.44699 16 6.36786 16 5.24264V5.05234C16 2.8143 14.1857 1 11.9477 1C10.7166 1 9.55233 1.55959 8.78331 2.52086L8 3.5L7.21669 2.52086C6.44767 1.55959 5.28338 1 4.05234 1C1.8143 1 0 2.8143 0 5.05234V5.24264C0 6.36786 0.44699 7.44699 1.24264 8.24264Z\" fill=\"#a6e3a1\"></path> </g></svg>";

pub async fn run() {
    let app = Router::new()
        .route("/", get(root))
        .route("/{user}", get(user))
        .route("/main.css", get(css));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", Config::get().port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<String> {
    let mut users = Storage::copy()
        .users
        .keys()
        .filter(|name| Config::get().keys.contains(&**name))
        .collect::<Vec<_>>();
    users.sort_by(
        |a, b| match (Storage::is_online(a), Storage::is_online(b)) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.cmp(b),
        },
    );
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Hypixel Screentime</title>
</head>
<body>
<style>
body {{
    margin: 0px;
    background: #333333;
    text-align: center;
    color: #eceff4 !important;
    font-family: Arial, Helvetica, sans-serif;
}}

.name {{
    padding: 8px;
    font-size: larger;
}}

.name a {{
    text-decoration: none;
}}

.name a:hover {{
    transition: 100ms;
    color: #ee99a0 !important;
}}

.name a {{
    color: #eceff4 !important;
    transition: 100ms;
}}

.green {{
    color: #a6e3a1 !important;
}}

#container {{
display:flex;
}}

#info {{
  flex:1;
  text-align: left;
  display: inline-block;
  align-content: center;
  margin-right: 100px;
}}

#players {{
  text-align: center;
}}
</style>

<div style="height: 100vh; display: flex; margin: 0px;">
<div style="flex: 1; align-content: center; display: inline-block;">

<!-- centering begin -->
<div style="margin: auto; display: inline-block; ">
<div id="container">
<div id="info" style="display: inline-block; max-width: fit-content !important; font-size: 1.5em;">
    <h1 style="text-shadow: 3px 3px #ce4008;">Hypixel Screentime</h1>
</div>
<div id="players" style="display: inline-block; max-width: fit-content; !important">
    {}
</div>
</div>
</div>

<br>
<br>
<footer>
  <p>Hypixel Screentime by <i><b>Sirius</b></i> | <span style="border-bottom: 2px solid #a6e3a1;"><a class="green" target="_blank" href="https://github.com/siriusmart/hypixel-screentime" style="text-decoration: none;">Written with {HEART} in Rust</a></span></p>
</footer>
<!-- centering end -->


</div>
</div>

</body>
</html>"#,
        users
            .iter()
            .map(|name| format!(
                r#"<div class="name"><a{} href="{name}">{name}{}</a></div>"#,
                if Storage::is_online(name) {
                    " class=\"green\""
                } else {
                    ""
                },
                if Storage::is_online(name) {
                    " (online)"
                } else {
                    ""
                }
            ))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

async fn user(Path((user,)): Path<(String,)>) -> Html<String> {
    Html(Mermaid::html(&user))
}

async fn css() -> String {
    fs::read_to_string("./main.css").unwrap()
}
