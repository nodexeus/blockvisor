// This will eventually be broken out into sub files in module

/// GET handler for health requests by an application platform
///
/// Intended for use in environments such as Amazon ECS or Kubernetes which want
/// to validate that the HTTP service is available for traffic, by returning a
/// 200 OK response with any content.
#[allow(clippy::unused_async)]
pub async fn health_endpoint() -> &'static str {
    "OK"
}
