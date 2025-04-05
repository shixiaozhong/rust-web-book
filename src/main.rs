#![warn(clippy::all)]

use handle_errors::return_error;
use warp::{http::Method, Filter};

mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() {
    // 创建内部共享存储
    let store = store::Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // cors，处理跨域问题
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    // get
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_questions);

    // put
    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    // delete
    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    // post add question
    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    // post add answer
    let add_answer = warp::post()
        .and(warp::path("comments"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    // 合并所有路由
    let routes = get_questions
        .or(update_question)
        .or(add_question)
        .or(add_answer)
        .or(delete_question)
        .with(cors)
        //为路由链中可能发生的错误（Rejection 或自定义错误）提供统一的恢复（Recovery）和转换逻辑
        .recover(return_error);

    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
