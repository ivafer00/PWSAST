use axum::response::{Html, IntoResponse, Response};
use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;
use log::info;
use crate::AppState;
use crate::controllers::html::scripts::scripts_controller;


pub fn router() -> Router<AppState> {
    Router::new()
        .merge(scripts_controller::router())
        .fallback(error404)
}
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "403.html")]
pub struct Error403Template {
    pub app_name: String,
}
#[derive(Template)]
#[template(path = "404.html")]
pub struct Error404Template {
    pub app_name: String,
}

pub async fn error404(State(state): State<AppState>) -> impl IntoResponse {
    info!("Couldn't find the requested page...");

    let version = state.config.app.name;

    let template = Error404Template { app_name: version };

    HtmlTemplate(template)
}

pub enum HTTPResponse<T> where
    T: Template, {
    OK200(T),
    FORBIDDEN403(Error403Template),
    NOTFOUND404(Error404Template)
}

impl<T> IntoResponse for HTTPResponse<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        match self {
            HTTPResponse::OK200(template) => HtmlTemplate(template).into_response(),
            HTTPResponse::FORBIDDEN403(template) => HtmlTemplate(template).into_response(),
            HTTPResponse::NOTFOUND404(template) => HtmlTemplate(template).into_response()
        }
    }
}