use std::{cmp::Ordering, fs};

use axum::{Router, extract::Path, response::Html, routing::get};

use crate::{Config, Storage, mermaid::Mermaid};

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
    let mut users = Storage::copy().users.keys().collect::<Vec<_>>();
    users.sort_by(|a, b| {
        if Storage::is_online(a) && !Storage::is_online(b) {
            Ordering::Less
        } else if Storage::is_online(b) {
            Ordering::Greater
        } else {
            a.cmp(b)
        }
    });
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
    background: #333333;
    text-align: center;
    color: #eceff4 !important;
    font-family: Arial, Helvetica, sans-serif;
}}

a {{
    color: #eceff4 !important;
    font-size: larger;
    text-decoration: none;
}}

.name {{
    padding: 3px;
}}

.green {{
    color: #a6e3a1 !important;
}}
</style>

<h1>Hypixel Screentime</h1>
{}
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
