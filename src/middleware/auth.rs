// // Authentication middleware for Axum
// use axum::{
//     extract::Request,
//     http::HeaderMap,
//     middleware::Next,
//     response::Response,
// };
// use tower::Layer;
// use std::task::{Context, Poll};
// use futures::future::BoxFuture;

// #[derive(Clone)]
// pub struct AuthLayer;

// impl<S> Layer<S> for AuthLayer {
//     type Service = AuthMiddleware<S>;

//     fn layer(&self, inner: S) -> Self::Service {
//         AuthMiddleware { inner }
//     }
// }

// #[derive(Clone)]
// pub struct AuthMiddleware<S> {
//     inner: S,
// }

// impl<S> tower::Service<Request> for AuthMiddleware<S>
// where
//     S: tower::Service<Request, Response = Response> + Send + 'static,
//     S::Future: Send + 'static,
// {
//     type Response = Response;
//     type Error = S::Error;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.inner.poll_ready(cx)
//     }

//     fn call(&mut self, mut req: Request) -> Self::Future {
//         let future = self.inner.call(req);

//         Box::pin(async move {
//             // TODO: Implement token validation logic
//             // - Extract Authorization header
//             // - Validate JWT token
//             // - Extract user_id from token
//             // - Store user_id in request extensions
            
//             let res = future.await?;
//             Ok(res)
//         })
//     }
// }

// // Helper middleware function for Axum
// pub async fn auth_middleware(
//     headers: HeaderMap,
//     mut req: Request,
//     next: Next,
// ) -> Response {
//     // TODO: Implement authentication
//     // - Extract and validate JWT from Authorization header
//     // - Store user_id in request extensions
    
//     next.run(req).await
// }
