//! Admin UI 路由配置

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Response, StatusCode, Uri, header},
    response::IntoResponse,
    routing::get,
};
use rust_embed::Embed;

/// 嵌入前端构建产物
#[derive(Embed)]
#[folder = "admin-ui/dist"]
struct Asset;

/// 创建 Admin UI 路由
pub fn create_admin_ui_router(base_path: String) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler))
        .with_state(base_path)
}

/// 处理首页请求
async fn index_handler(State(base_path): State<String>) -> impl IntoResponse {
    serve_index(&base_path)
}

/// 处理静态文件请求
async fn static_handler(State(base_path): State<String>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // 安全检查：拒绝包含 .. 的路径
    if path.contains("..") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Invalid path"))
            .expect("Failed to build response");
    }

    // 尝试获取请求的文件
    if let Some(content) = Asset::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // 根据文件类型设置不同的缓存策略
        let cache_control = get_cache_control(path);

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, cache_control)
            .body(Body::from(content.data.into_owned()))
            .expect("Failed to build response");
    }

    // SPA fallback: 如果文件不存在且不是资源文件，返回 index.html
    if !is_asset_path(path) {
        return serve_index(&base_path);
    }

    // 404
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not found"))
        .expect("Failed to build response")
}

/// 提供 index.html（注入运行时配置）
fn serve_index(base_path: &str) -> Response<Body> {
    match Asset::get("index.html") {
        Some(content) => {
            let html = String::from_utf8_lossy(&content.data);

            // 注入运行时配置
            let config_script = format!(
                r#"<script>window.__KIRO_CONFIG__={{basePath:"{}"}}</script>"#,
                base_path
            );
            let modified_html = html.replace("</head>", &format!("{}</head>", config_script));

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(header::CACHE_CONTROL, "no-cache")
                .body(Body::from(modified_html))
                .expect("Failed to build response")
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(
                "Admin UI not built. Run 'pnpm build' in admin-ui directory.",
            ))
            .expect("Failed to build response"),
    }
}

/// 根据文件类型返回合适的缓存策略
fn get_cache_control(path: &str) -> &'static str {
    if path.ends_with(".html") {
        // HTML 文件不缓存，确保用户获取最新版本
        "no-cache"
    } else if path.starts_with("assets/") {
        // assets/ 目录下的文件带有内容哈希，可以长期缓存
        "public, max-age=31536000, immutable"
    } else {
        // 其他文件（如 favicon）使用较短的缓存
        "public, max-age=3600"
    }
}

/// 判断是否为资源文件路径（有扩展名的文件）
fn is_asset_path(path: &str) -> bool {
    // 检查最后一个路径段是否包含扩展名
    path.rsplit('/')
        .next()
        .map(|filename| filename.contains('.'))
        .unwrap_or(false)
}
