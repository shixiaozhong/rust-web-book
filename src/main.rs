#![warn(clippy::all)]

use handle_errors::return_error;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() {
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // log::error!("This is an error");
    // log::debug!("This is debug");
    // log::info!("This is info");

    // // 自定义log
    // let log = warp::log::custom(|info| {
    //     log::info!(
    //         "{} {} {} {:?} from {} with {:?}",
    //         info.method(),
    //         info.path(),
    //         info.status(),
    //         // 获取请求时间
    //         info.elapsed(),
    //         // 获取请求来源
    //         info.remote_addr().unwrap(),
    //         // 获取请求头
    //         info.request_headers()
    //     );
    // });

    // 定义一个log_filter
    let log_filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "rust_web_book=info,warp=error".to_owned());

    tracing_subscriber::fmt()
        // 使用上面的过滤器来决定记录哪些追踪
        .with_env_filter(log_filter)
        // 在每个span关闭时记录event，可以用来记录路由执行的时间
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // 创建内部共享存储
    let store = store::Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // 创建一个uuid
    // let id_filter = warp::any().map(|| uuid::Uuid::new_v4().to_string());

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
        .and_then(routes::question::get_questions)
        .with(warp::trace(|info| {
            tracing::info_span!("get_questions request", method = %info.method(), path = %info.path(), id = %uuid::Uuid::new_v4())
        }));

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
        .with(warp::trace::request())
        //为路由链中可能发生的错误（Rejection 或自定义错误）提供统一的恢复（Recovery）和转换逻辑
        .recover(return_error);

    // 启动服务
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
