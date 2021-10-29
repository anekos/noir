use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, http, web};
use log::info;
use serde::{Deserialize, Serialize};

use crate::alias::Alias;
use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::expander::Expander;
use crate::global_alias::GlobalAliasTable;
use crate::meta::Meta;
use crate::search_history::SearchHistory;
use crate::tag::Tag;

pub mod download;
pub mod util;


pub struct AppData {
    pub aliases: GlobalAliasTable,
    pub db: Database,
    pub dl_manager:  download::Manager,
    pub download_to: Option<String>,
}

#[derive(Deserialize)]
struct FileQuery {
    path: String
}

#[derive(Deserialize)]
struct FileTagsQuery {
    path: String
}

#[derive(Deserialize)]
struct SearchQuery {
    expression: String,
    record: Option<bool>
}

#[derive(Serialize)]
struct QueryResult {
    items: Vec<Meta>,
    expression: String,
}

#[derive(Deserialize)]
struct DownloadRequest {
    tags: Option<download::Tags>,
    to: String,
    url: String,
}

#[derive(Deserialize)]
struct SetTagRequest {
    path: String,
    tags: download::Tags,
}

async fn on_alias(data: web::Data<Mutex<AppData>>, name: web::Path<String>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let alias = expander.get_alias(&name);
    Ok(HttpResponse::Ok().json(alias))
}

async fn on_alias_delete(data: web::Data<Mutex<AppData>>, name: web::Path<String>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock delete alias");
    let _tx = data.db.transaction()?;
    data.db.delete_alias(&name)?;
    Ok(HttpResponse::Ok().json(true))
}

async fn on_alias_update(data: web::Data<Mutex<AppData>>, name: web::Path<String>, alias: web::Json<Alias>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock set alias");
    let _tx = data.db.transaction()?;
    data.db.upsert_alias(&name, &alias.expression, alias.recursive)?;
    Ok(HttpResponse::Ok().json(true))
}

async fn on_aliases(data: web::Data<Mutex<AppData>>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let aliases: Vec<&str> = expander.get_alias_names();
    Ok(HttpResponse::Ok().json(aliases))
}

async fn on_download(data: web::Data<Mutex<AppData>>, request: web::Json<DownloadRequest>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock downlod");

    if let Some(download_to) = &data.download_to {
        let mut to = Path::new(&download_to).to_path_buf();
        let suffix = util::shorten_path(&request.to);
        to.push(&suffix);

        let job = download::Job {
            to,
            tags: request.tags.clone(),
            url: request.url.clone(),
        };
        data.dl_manager.download(job);

        return Ok(HttpResponse::Ok().json(true))
    }

    Err(AppError::Standard("Server option `download-to` is not given"))
}

async fn on_file(data: web::Data<Mutex<AppData>>, query: web::Query<FileQuery>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock file");
    let found = data.db.get(&query.path)?;
    info!("on_file: file={:?}", query.path);
    let found = found.ok_or(AppError::Void)?;
    let mut content: Vec<u8> = vec![];
    let mut file = File::open(&found.file.path)?;
    file.read_to_end(&mut content)?;
    let content_type = format!("image/{}", found.format);
    Ok(
        HttpResponse::Ok()
        .header("Cache-Control", "public,immutable,max-age=3600")
        .content_type(content_type)
        .body(content)
    )
}

async fn on_file_tags(data: web::Data<Mutex<AppData>>, query: web::Query<FileTagsQuery>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock file tags");
    let tags = data.db.tags_by_path(&query.path)?;
    info!("on_file_tags: file={:?}, tags={:?}", query.path, tags);
    Ok(HttpResponse::Ok().json(tags))
}

async fn on_history(data: web::Data<Mutex<AppData>>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let history: Vec<SearchHistory> = data.db.search_history()?;
    Ok(HttpResponse::Ok().json(history))
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

    if query.record.unwrap_or(false) {
        data.db.add_search_history(&query.expression)?;
    }

    Ok(HttpResponse::Ok().json(QueryResult { items, expression }))
}

async fn on_set_tags(data: web::Data<Mutex<AppData>>, request: web::Json<SetTagRequest>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock set_tag");
    let mut tags = vec![];
    for tag in &request.tags.items {
        tags.push(Tag::from_str(&tag)?);
    }
    data.db.add_tags(&request.path, &tags, &request.tags.source)?;
    Ok(HttpResponse::Ok().json(true))
}

async fn on_tags(data: web::Data<Mutex<AppData>>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let tags: Vec<&str> = expander.get_tag_names();
    Ok(HttpResponse::Ok().json(tags))
}

#[actix_web::main]
pub async fn start(
    db: Database,
    dl_manager: download::Manager,
    aliases: GlobalAliasTable,
    port: u16,
    root: String,
    download_to: Option<String>
) -> std::io::Result<()> {

    let app_data = AppData { aliases, dl_manager, db, download_to };
    let data = web::Data::new(Mutex::new(app_data));

    HttpServer::new(move || {
        let cors = Cors::default()
             .allow_any_origin()
             .allowed_methods(vec!["GET", "POST", "DELETE"])
             .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
             .allowed_header(http::header::CONTENT_TYPE)
             .max_age(3600);
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(data.clone())
            .service(web::resource("/alias/{name}")
                .route(web::get().to(on_alias))
                .route(web::delete().to(on_alias_delete))
                .route(web::post().to(on_alias_update)))
            .service(web::resource("/aliases").route(web::get().to(on_aliases)))
            .service(web::resource("/download").route(web::post().to(on_download)))
            .service(web::resource("/file").route(web::get().to(on_file)))
            .service(web::resource("/file/tags").route(web::get().to(on_file_tags)))
            .service(web::resource("/history").route(web::get().to(on_history)))
            .service(web::resource("/search").route(web::post().to(on_search)))
            .service(
                web::resource("/tags")
                .route(web::get().to(on_tags))
                .route(web::post().to(on_set_tags)))
            .service(Files::new("/", &root).index_file("index.html"))
    }).bind(("0.0.0.0", port))?.run().await
}
