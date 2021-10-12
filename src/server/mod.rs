use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, http, web};
use serde::{Deserialize, Serialize};

use crate::alias::Alias;
use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::expander::Expander;
use crate::global_alias::GlobalAliasTable;
use crate::loader;
use crate::meta::Meta;
use crate::search_history::SearchHistory;
use crate::tag::Tag;

mod download;


struct AppData {
    aliases: GlobalAliasTable,
    db: Database,
    download_to: Option<String>,
}

#[derive(Deserialize)]
struct FileQuery {
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
    tags: Option<Vec<String>>,
    to: String,
    url: String,
}

async fn on_alias(data: web::Data<Mutex<AppData>>, name: web::Path<String>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let alias = expander.get_alias(&name);
    Ok(HttpResponse::Ok().json(alias))
}

async fn on_alias_delete(data: web::Data<Mutex<AppData>>, name: web::Path<String>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock delete alias");
    data.db.delete_alias(&name)?;
    data.db.flush()?;
    Ok(HttpResponse::Ok().json(true))
}

async fn on_alias_update(data: web::Data<Mutex<AppData>>, name: web::Path<String>, alias: web::Json<Alias>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock set alias");
    data.db.upsert_alias(&name, &alias.expression, alias.recursive)?;
    data.db.flush()?;
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
        let mut download_to = Path::new(&download_to).to_path_buf();
        download_to.push(&request.to);

        download::download(&request.url, &download_to)?;

        let config = loader::Config::default();
        let mut loader = loader::Loader::new(&data.db, config);
        loader.load_file(&download_to)?;
        if let Some(ref tags) = request.tags {
            let mut _tags = vec![];
            for tag in tags {
                _tags.push(Tag::from_str(&tag)?);
            }
            let download_to = download_to.to_str().unwrap();
            data.db.add_tags(&download_to, &_tags)?;
        }
        data.db.flush()?;

        return Ok(HttpResponse::Ok().json("ok"))
    }

    Err(AppError::Standard("`download-to` option is not given"))
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

async fn on_tags(data: web::Data<Mutex<AppData>>) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock search");
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let tags: Vec<&str> = expander.get_tag_names();
    Ok(HttpResponse::Ok().json(tags))
}

#[actix_web::main]
pub async fn start(db: Database, aliases: GlobalAliasTable, port: u16, root: String, download_to: Option<String>) -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppData{aliases, db, download_to}));

    HttpServer::new(move || {
        let cors = Cors::default()
             .allow_any_origin()
             .allowed_methods(vec!["GET", "POST", "DELETE"])
             .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
             .allowed_header(http::header::CONTENT_TYPE)
             .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(data.clone())
            .service(web::resource("/alias/{name}")
                .route(web::get().to(on_alias))
                .route(web::delete().to(on_alias_delete))
                .route(web::post().to(on_alias_update)))
            .service(web::resource("/aliases").route(web::get().to(on_aliases)))
            .service(web::resource("/download").route(web::post().to(on_download)))
            .service(web::resource("/file").route(web::get().to(on_file)))
            .service(web::resource("/history").route(web::get().to(on_history)))
            .service(web::resource("/search").route(web::post().to(on_search)))
            .service(web::resource("/tags").route(web::get().to(on_tags)))
            .service(Files::new("/", &root).index_file("index.html"))
    }).bind(("0.0.0.0", port))?.run().await
}
