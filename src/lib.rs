use lazy_static::lazy_static;
use std::collections::HashMap;
use worker::*;

lazy_static! {
    static ref EAS_CHAIN_GQL_ENDPOINT: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("optimism", "https://optimism.easscan.org/graphql");
        m.insert("sepolia", "https://sepolia.easscan.org/graphql");
        m.insert("base", "https://base.easscan.org/graphql");
        m
    };
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    let router = Router::new();
    router
        .get_async("/graphql/:chain/:cache_key", |req, ctx| async move {
            handle_graphql_request(req, ctx).await
        })
        .post_async("/graphql/:chain/:cache_key", |req, ctx| async move {
            handle_graphql_request(req, ctx).await
        })
        .run(req, env)
        .await
}

pub async fn handle_graphql_request(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    if let Some(chain) = ctx.param("chain") {
        if ctx.param("cache_key").is_some() {
            if let Some(uri) = EAS_CHAIN_GQL_ENDPOINT.get(chain.as_str()) {
                let url = req.url()?;

                let c = Cache::default();
                let cached = c.get(url.as_str(), false).await?;
                if let Some(response) = cached {
                    return Ok(response);
                }

                let body = req.text().await?;

                let mut headers = Headers::new();
                headers.append("Content-Type", "application/json")?;
                headers.append("User-Agent", "c-atts/0.0.1")?;

                let mut init = RequestInit::new();
                init.with_headers(headers);
                init.with_method(Method::Post);
                init.with_body(Some(body.into()));

                let request = Request::new_with_init(uri, &init)?;
                let response = Fetch::Request(request).send().await;

                match response {
                    Ok(mut response) => {
                        let cloned_response = response.cloned()?;
                        c.put(url.as_str(), cloned_response).await?;
                        return Ok(response);
                    }
                    Err(e) => return Response::error(format!("Error fetching data: {}", e), 500),
                }
            }
            return Response::error("Chain not supported", 400);
        }
        return Response::error("Cache key parameter missing", 404);
    }
    Response::error("Chain parameter missing", 404)
}
