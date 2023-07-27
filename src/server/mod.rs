use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, http, web};
use log::{info, error};
use logging_timer::{timer, executing, Level};
use serde::{Deserialize, Serialize};

use crate::alias::Alias;
use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::expander::Expander;
use crate::expression::modifier::replace_tag;
use crate::expression::parser::parse;
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
struct Favorite {
    path: String,
    toggle: Option<bool>
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
struct ExpressionReplaceTag {
    expression: String,
    tag: String
}

#[derive(Deserialize)]
struct SetTagRequest {
    path: String,
    tags: download::Tags,
}

fn update_favorite(
    data: web::Data<Mutex<AppData>>,
    favorite: web::Query<Favorite>,
    tag_to_add: &'static str,
    tags_to_delete: &'static[&'static str]
) -> AppResult<HttpResponse> {
    let data = data.lock().expect("lock file");

    {
        let mut tags = vec![];
        for tag in tags_to_delete {
            tags.push(Tag::from_str(tag)?);
        }
        data.db.delete_tags(&favorite.path, &tags, "noir")?;
    }

    if let Some(toggle) = favorite.toggle {
        if toggle && data.db.tag_exists(&favorite.path, tag_to_add)? {
            let tags = [Tag::from_str(tag_to_add)?];
            data.db.delete_tags(&favorite.path, &tags, "noir")?;
            return Ok(HttpResponse::Ok().json(false))
        }
    }

    let tags = [Tag::from_str(tag_to_add)?];
    data.db.add_tags(&favorite.path, &tags, "noir")?;

    Ok(HttpResponse::Ok().json(true))
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

        let job_json = serde_json::to_string(&job)?;
        let _tx = data.db.transaction()?;
        data.db.queue(&request.url, &job_json)?;

        if data.dl_manager.download(job) {
            return Ok(HttpResponse::Ok().json(true))
        } else {
            error!("Failed to download: mpsc error");
            return Ok(HttpResponse::InternalServerError().json("Failed to download: mpsc error"))
        }
    }

    Err(AppError::Standard("Server option `download-to` is not given"))
}

async fn on_dislike(data: web::Data<Mutex<AppData>>, favorite: web::Query<Favorite>) -> AppResult<HttpResponse> {
    update_favorite(data, favorite, "dislike", &["like", "neutral"])
}

async fn on_expression_replace_tag(query: web::Json<ExpressionReplaceTag>) -> AppResult<HttpResponse> {
    let q = parse(&query.expression)?;
    let expression = replace_tag(q, &query.tag)?;
    Ok(HttpResponse::Ok().json(expression.map(|it| it.to_string())))
}

async fn on_file(data: web::Data<Mutex<AppData>>, query: web::Query<FileQuery>) -> AppResult<HttpResponse> {
    let timer = timer!(Level::Info; "on_file_tags");

    let data = data.lock().expect("lock file");

    executing!(timer, "Get meta from database: path={}", query.path);
    let found = data.db.get(&query.path)?;
    let found = found.ok_or(AppError::Void)?;

    let mut content: Vec<u8> = vec![];
    executing!(timer, "Read file: path={}", query.path);
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
    let _timer = timer!(Level::Info; "on_file_tags");
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

async fn on_like(data: web::Data<Mutex<AppData>>, favorite: web::Query<Favorite>) -> AppResult<HttpResponse> {
    update_favorite(data, favorite, "like", &["dislike", "neutral"])
}

async fn on_neutral(data: web::Data<Mutex<AppData>>, favorite: web::Query<Favorite>) -> AppResult<HttpResponse> {
    update_favorite(data, favorite, "neutral", &["like", "dislike"])
}

async fn on_search(data: web::Data<Mutex<AppData>>, query: web::Json<SearchQuery>) -> AppResult<HttpResponse> {
    let timer = timer!(Level::Info; "on_search");

    let data = data.lock().expect("lock search");

    executing!(timer, "Expand: {}", &query.expression);
    let expander = Expander::generate(&data.db, &data.aliases)?;
    let expression = expander.expand_str(&query.expression)?;
    info!("on_search: raw_expression={:?}", expression);

    executing!(timer, "Search from database: {}", &query.expression);
    let mut items: Vec<Meta> = vec![];
    data.db.select(expression.as_ref(), false, |meta, _vacuumed| {
        items.push(meta.clone());
        Ok(())
    })?;

    executing!(timer, "Add history: {}", &query.expression);
    if query.record.unwrap_or(false) {
        data.db.add_search_history(&query.expression)?;
    }

    Ok(HttpResponse::Ok().json(QueryResult { items, expression: expression.to_string() }))
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
    let tags: Vec<String> = data.db.tags()?;
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
            .service(web::resource("/like").route(web::post().to(on_like)))
            .service(web::resource("/dislike").route(web::post().to(on_dislike)))
            .service(web::resource("/neutral").route(web::post().to(on_neutral)))
            .service(web::resource("/file").route(web::get().to(on_file)))
            .service(web::resource("/file/tags").route(web::get().to(on_file_tags)))
            .service(web::resource("/history").route(web::get().to(on_history)))
            .service(web::resource("/search").route(web::post().to(on_search)))
            .service(web::resource("/expression/replace_tag").route(web::post().to(on_expression_replace_tag)))
            .service(
                web::resource("/tags")
                .route(web::get().to(on_tags))
                .route(web::post().to(on_set_tags)))
            .service(Files::new("/", &root).index_file("index.html"))
    }).bind(("0.0.0.0", port))?.run().await
}
