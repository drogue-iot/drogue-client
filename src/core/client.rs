#[cfg(feature = "telemetry")]
pub use self::tracing::*;

#[cfg(feature = "telemetry")]
mod tracing {
    use http::HeaderMap;
    use opentelemetry::propagation::Injector;
    use opentelemetry::Context;
    use reqwest::RequestBuilder;

    pub trait WithTracing {
        fn propagate_context(self, cx: &Context) -> Self;

        fn propagate_current_context(self) -> Self
        where
            Self: Sized,
        {
            self.propagate_context(&Context::current())
        }
    }

    impl WithTracing for RequestBuilder {
        fn propagate_context(self, cx: &Context) -> Self {
            let headers = opentelemetry::global::get_text_map_propagator(|prop| {
                let mut injector = HeaderInjector::new();
                prop.inject_context(&cx, &mut injector);
                injector.0
            });
            self.headers(headers)
        }
    }

    struct HeaderInjector(HeaderMap);

    impl HeaderInjector {
        pub fn new() -> Self {
            Self(Default::default())
        }
    }

    impl Injector for HeaderInjector {
        /// Set a key and value in the HeaderMap.  Does nothing if the key or value are not valid inputs.
        fn set(&mut self, key: &str, value: String) {
            if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                    self.0.insert(name, val);
                }
            }
        }
    }
}
