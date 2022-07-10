use std::{
    collections::HashMap,
    thread, // Use Thread for spawning a thread e.g. to acquire our DATA mutex lock.
};
use std::net::SocketAddr;

use axum::{
    extract::{
        Form,
        Json,
        Path,
        Query,
    },
    handler::Handler,
    http::{
        header,
        StatusCode,
        Uri,
    },
    response::{
        AppendHeaders,
        Html,
    },
    Router,
    routing::get,
    Server,
};
/// Use Serde JSON to serialize/deserialize JSON, such as in a request.
/// axum creates JSON or extracts it by using `axum::extract::Json`.
/// For this demo, see functions `get_demo_json` and `post_demo_json`.
use serde_json::{
    json,
    Value,
};
use tracing_subscriber::{
    fmt::layer,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::book::Book;
use crate::data::DATA;

mod book;
mod data;

#[tokio::main]
pub async fn main() {
    let host = [0, 0, 0, 0];
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from((host, port));
    // Start tracing.
    tracing_subscriber::registry()
        .with(layer())
        .init();
    let app = Router::new()
        .route("/", get(hello))
        .route("/demo.html", get(get_demo_html))
        .route("/hello.html", get(hello_html))
        .route("/demo-status", get(demo_status))
        .route("/demo-uri", get(demo_uri))
        .route("/demo.png", get(get_demo_png))
        .route("/foo",
               get(get_foo)
                   .put(put_foo)
                   .patch(patch_foo)
                   .post(post_foo)
                   .delete(delete_foo))
        .route("/items/:id", get(get_items_id))
        .route("/items", get(get_items))
        .route("/demo.json",
               get(get_demo_json)
                   .put(put_demo_json))
        .route("/books",
               get(get_books)
                   .put(put_books))
        .route("/books/:id",
               get(get_books_id)
                   .delete(delete_books_id))
        .route("/books/:id/form",
               get(get_books_id_form)
                   .post(post_books_id_form),
        )
        .fallback(fallback.into_service());
    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// axum handler for "DELETE /books/:id" which destroys a resource.
/// This demo extracts an id, then mutates the book in the DATA store.
pub async fn delete_books_id(Path(id): Path<u32>) -> Html<String> {
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        if data.contains_key(&id) {
            data.remove(&id);
            format!("Delete book id: {}", &id)
        } else {
            format!("Book id not found: {}", &id)
        }
    }).join().unwrap().into()
}

/// axum handler for "POST /books/:id/form" which submits an HTML form.
/// This demo shows how to do a form submission then update a resource.
pub async fn post_books_id_form(form: Form<Book>) -> Html<String> {
    let new_book: Book = form.0;
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        if data.contains_key(&new_book.id) {
            data.insert(new_book.id, new_book.clone());
            format!("<p>{}</p>\n", &new_book)
        } else {
            format!("Book id not found: {}", &new_book.id)
        }
    }).join().unwrap().into()
}

/// axum handler for "GET /books/:id/form" which responds with a form.
/// This demo shows how to write a typical HTML form with input fields.
pub async fn get_books_id_form(Path(id): Path<u32>) -> Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        match data.get(&id) {
            Some(book) => format!(
                concat!(
                "<form method=\"post\" action=\"/books/{}/form\">\n",
                "<input type=\"hidden\" name=\"id\" value=\"{}\">\n",
                "<p><input name=\"title\" value=\"{}\"></p>\n",
                "<p><input name=\"author\" value=\"{}\"></p>\n",
                "<input type=\"submit\" value=\"Save\">\n",
                "</form>\n"
                ),
                &book.id,
                &book.id,
                &book.title,
                &book.author
            ),
            None => format!("<p>Book id {} not found</p>", id),
        }
    }).join().unwrap().into()
}

/// axum handler for "PUT /books" which creates a new book resource.
/// This demo shows how axum can extract JSON data into a Book struct.
pub async fn put_books(Json(book): Json<Book>) -> Html<String> {
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        data.insert(book.id, book.clone());
        format!("Put book: {}", &book)
    }).join().unwrap().into()
}

/// axum handler for "GET /books/:id" which responds with one resource HTML page.
/// This demo app uses our DATA variable, and iterates on it to find the id.
pub async fn get_books_id(Path(id): Path<u32>) -> Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        match data.get(&id) {
            Some(book) => format!("<p>{}</p>\n", &book),
            None => format!("<p>Book id {} not found</p>", id),
        }
    }).join().unwrap().into()
}

/// axum handler for "GET /books" which responds with a resource page.
/// This demo uses our DATA; a production app could use a database.
/// This demo must clone the DATA in order to sort items by title.
pub async fn get_books() -> Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        let mut books = data.values()
            .collect::<Vec<_>>()
            .clone();
        books.sort_by(|a, b| a.title.cmp(&b.title));
        books.iter()
            .map(|&book| format!("<p>{}</p>\n", &book))
            .collect::<String>()
    }).join().unwrap().into()
}

/// To access data, create a thread, spawn it, then get the lock.
/// When you're done, then join the thread with its parent thread.
// async fn print_data() {
//     thread::spawn(move || {
//         let data = DATA.lock().unwrap();
//         println!("data: {:?}", data);
//     }).join().unwrap()
// }

/// axum handler for "PUT /demo.json" which uses `aumx::extract::Json`.
/// This buffers the request body then deserializes it using serde.
/// The `Json` type supports types that implement `serde::Deserialize`.
pub async fn put_demo_json(Json(data): Json<Value>) -> String {
    format!("Put demo JSON data: {:?}", data)
}

/// axum handler for "PUT /demo.json" which uses `aumx::extract::Json`.
/// This buffers the request body then deserializes it bu using serde.
/// The `Json` type supports types that implement `serde::Deserialize`.
pub async fn get_demo_json() -> Json<Value> {
    json!({"a":"b"}).into()
}

/// axum handler for "GET /items" which uses `axum::extract::Query`.
/// This extracts query parameters and creates a key-value pair map.
pub async fn get_items(Query(params): Query<HashMap<String, String>>) -> String {
    format!("Get items with query params: {:?}", params)
}

/// axum handler for "GET /items/:id" which uses `axum::extract::Path`.
/// This extracts a path parameter then deserializes it as needed.
pub async fn get_items_id(Path(id): Path<String>) -> String {
    format!("Get items with path id: {:?}", id)
}

/// axum handler for "GET /foo" which returns a string message.
/// This shows our naming convention for HTTP GET handlers.
pub async fn get_foo() -> String {
    "GET foo".to_string()
}

/// axum handler for "PUT /foo" which returns a string message.
/// This shows our naming convention for HTTP PUT handlers.
pub async fn put_foo() -> String {
    "PUT foo".to_string()
}

/// axum handler for "PATCH /foo" which returns a string message.
/// This shows our naming convention for HTTP PATCH handlers.
pub async fn patch_foo() -> String {
    "PATCH foo".to_string()
}

/// axum handler for "POST /foo" which returns a string message.
/// This shows our naming convention for HTTP POST handlers.
pub async fn post_foo() -> String {
    "POST foo".to_string()
}

/// axum handler for "DELETE /foo" which returns a string message.
/// This shows our naming convention for HTTP DELETE handlers.
pub async fn delete_foo() -> String {
    "DELETE foo".to_string()
}

/// axum handler for "GET /" which returns a string and causes axum to
/// immediately respond with status code `200 OK` and with the string.
pub async fn hello() -> String {
    "Hello, World!".into()
}

/// axum handler for "GET /demo.html" which responds with HTML text.
/// The `Html` type sets an HTTP header content-type of `text/html`.
pub async fn get_demo_html() -> Html<&'static str> {
    "<h1>Hello</h1>".into()
}

/// axum handler that responds with typical HTML coming from a file.
/// This uses the Rust macro `std::include_str` to include a UTF-8 file
/// path, relative to `main.rs`, as a `&'static str` at compile time.
async fn hello_html() -> Html<&'static str> {
    include_str!("hello.html").into()
}

/// axum handler for "GET /demo-status" which returns a HTTP status
/// code, such as OK (200), and a custom user-visible string message.
pub async fn demo_status() -> (StatusCode, String) {
    (StatusCode::OK, "Everything is OK".to_string())
}

/// axum handler for "GET /demo-uri" which shows the request's own URI.
/// This shows how to write a handler that receives the URI.
pub async fn demo_uri(uri: Uri) -> String {
    format!("The URI is: {:?}", uri)
}

/// axum handler for "GET /demo.png" which responds with an image PNG.
/// This sets a header "image/png" then sends the decoded image data.
async fn get_demo_png() -> impl axum::response::IntoResponse {
    let png = concat!(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB",
    "CAYAAAAfFcSJAAAADUlEQVR42mPk+89Q",
    "DwADvgGOSHzRgAAAAABJRU5ErkJggg=="
    );
    (
        AppendHeaders([(header::CONTENT_TYPE, "image/png"), ]),
        base64::decode(png).unwrap(),
    )
}

/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
pub async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route {}", uri))
}

/// Tokio signal handler that will wait for a user to press CTRL+C.
/// We use this in our hyper `Server` method `with_graceful_shutdown`.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    println!("signal shutdown");
}