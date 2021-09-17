use std::fs::File;
use std::io::Read;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, http, web};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::expander::Expander;
use crate::global_alias::GlobalAliasTable;
use crate::meta::Meta;


struct AppData {
    db: Database,
    aliases: GlobalAliasTable,
}

#[derive(Deserialize)]
struct FileQuery {
    path: String
}

#[derive(Deserialize)]
struct SearchQuery {
    expression: String
}

#[derive(Serialize)]
struct QueryResult {
    items: Vec<Meta>,
    expression: String,
}

async fn on_aliases(data: web::Data<Mutex<AppData>>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let aliases: Vec<&str> = expander.get_alias_names();
    Ok(HttpResponse::Ok().json(aliases))
}

async fn on_tags(data: web::Data<Mutex<AppData>>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let tags: Vec<&str> = expander.get_tag_names();
    Ok(HttpResponse::Ok().json(tags))
}

async fn on_file(data: web::Data<Mutex<AppData>>, query: web::Query<FileQuery>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock file");
    let found = data.db.get(&query.path)?;
    let found = found.ok_or(AppError::Void)?;
    let mut content: Vec<u8> = vec![];
    let mut file = File::open(&found.file.path)?;
    file.read_to_end(&mut content)?;
    let content_type = format!("image/{}", found.format);
    Ok(HttpResponse::Ok().content_type(content_type).body(content))
}

async fn on_search(data: web::Data<Mutex<AppData>>, query: web::Json<SearchQuery>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let expression = expander.expand(&query.expression);

    let mut items: Vec<Meta> = vec![];

    data.db.select(&expression, false, |meta, _vacuumed| {
        items.push(meta.clone());
        Ok(())
    })?;

    data.db.add_search_history(&query.expression)?;

    Ok(HttpResponse::Ok().json(QueryResult { items, expression }))
}

#[actix_web::main]
pub async fn start(db: Database, aliases: GlobalAliasTable, port: u16, root: String) -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppData{aliases, db}));

    HttpServer::new(move || {
        let cors = Cors::default()
             .allow_any_origin()
             .allowed_methods(vec!["GET", "POST"])
             .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
             .allowed_header(http::header::CONTENT_TYPE)
             .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(data.clone())
            .service(web::resource("/search").route(web::post().to(on_search)))
            .service(web::resource("/aliases").route(web::get().to(on_aliases)))
            .service(web::resource("/tags").route(web::get().to(on_tags)))
            .service(web::resource("/file").route(web::get().to(on_file)))
            .service(Files::new("/", &root).index_file("index.html"))
    }).bind(("0.0.0.0", port))?.run().await
}