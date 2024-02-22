use serde::{Deserialize, Serialize};
use worker::*;
#[derive(Deserialize)]
struct ContentTypeCheckRequest {
    url: String,
    require_types: Option<Vec<String>>, // priority
    require_main_types: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ContentTypeCheckResponse {
    is_ok: bool,
    content_type: String,
    // error_message: Option<String>,
}

async fn get_content_type(url: &str) -> Result<String> {
    let request = Request::new_with_init(url, RequestInit::new().with_method(Method::Head))?;
    let resp = Fetch::Request(request).send().await?;
    let headers = resp.headers();
    let content_type = headers.get("content-type")?.unwrap_or_default();
    Ok(content_type)
}

#[event(fetch)]
async fn main(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    if let Method::Post = req.method() {
        let cr = match req.json::<ContentTypeCheckRequest>().await {
            Ok(r) => r,
            Err(e) => {
                return Response::error(format!("Failed to parse request: {}", e), 400);
            }
        };
        println!("Checking content type for {}", cr.url);
        let content_type = match get_content_type(&cr.url).await {
            Ok(ct) => ct,
            Err(e) => {
                return Response::error(format!("Failed to get content type: {}", e), 500);
            }
        };
        if let Some(types) = cr.require_types {
            for t in types {
                if content_type == t {
                    return Response::from_json::<ContentTypeCheckResponse>(
                        &ContentTypeCheckResponse {
                            is_ok: true,
                            content_type,
                        },
                    );
                }
            }
        } else if let Some(main_types) = cr.require_main_types {
            let main_type = content_type.split('/').next().unwrap_or_default();
            for t in main_types {
                if main_type == t {
                    return Response::from_json::<ContentTypeCheckResponse>(
                        &ContentTypeCheckResponse {
                            is_ok: true,
                            content_type,
                        },
                    );
                }
            }
        } else {
            return Response::error("No content type requirements provided", 400);
        }
        Response::from_json::<ContentTypeCheckResponse>(&ContentTypeCheckResponse {
            is_ok: false,
            content_type,
        })
    } else {
        Response::error("Unsupported method.", 405)
    }
}
