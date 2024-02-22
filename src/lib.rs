use serde::{Deserialize, Serialize};
use worker::*;
#[derive(Deserialize)]
struct ContentTypeCheckRequest {
    url: String,
    expected_types: Option<Vec<String>>, // priority
    expected_main_types: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ContentTypeCheckResponse {
    is_ok: bool,
    content_type: String,
    expected_types: Option<Vec<String>>,
    expected_main_types: Option<Vec<String>>,
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
        // check exact types first
        if let Some(ref types) = cr.expected_types {
            if !types.is_empty() {
                for t in types {
                    if &content_type == t {
                        return Response::from_json::<ContentTypeCheckResponse>(
                            &ContentTypeCheckResponse {
                                is_ok: true,
                                content_type,
                                expected_types: cr.expected_types,
                                expected_main_types: cr.expected_main_types,
                            },
                        );
                    }
                }
            }
        }
        // check main types
        if let Some(ref main_types) = cr.expected_main_types {
            if !main_types.is_empty() {
                let main_type = content_type.split('/').next().unwrap_or_default();
                for t in main_types {
                    if main_type == t {
                        return Response::from_json::<ContentTypeCheckResponse>(
                            &ContentTypeCheckResponse {
                                is_ok: true,
                                content_type,
                                expected_types: cr.expected_types,
                                expected_main_types: cr.expected_main_types,
                            },
                        );
                    }
                }
            }
        }

        Response::from_json::<ContentTypeCheckResponse>(&ContentTypeCheckResponse {
            is_ok: false,
            content_type,
            expected_types: cr.expected_types,
            expected_main_types: cr.expected_main_types,
        })
    } else {
        Response::error("Unsupported method.", 405)
    }
}
