(function() {var implementors = {
"axum":[["impl&lt;M, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.0/core/future/into_future/trait.IntoFuture.html\" title=\"trait core::future::into_future::IntoFuture\">IntoFuture</a> for <a class=\"struct\" href=\"axum/serve/struct.Serve.html\" title=\"struct axum::serve::Serve\">Serve</a>&lt;M, S&gt;<span class=\"where fmt-newline\">where\n    M: for&lt;'a&gt; <a class=\"trait\" href=\"tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"axum/serve/struct.IncomingStream.html\" title=\"struct axum::serve::IncomingStream\">IncomingStream</a>&lt;'a&gt;, Error = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.74.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, Response = S&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,\n    for&lt;'a&gt; &lt;M as <a class=\"trait\" href=\"tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"axum/serve/struct.IncomingStream.html\" title=\"struct axum::serve::IncomingStream\">IncomingStream</a>&lt;'a&gt;&gt;&gt;::<a class=\"associatedtype\" href=\"tower_service/trait.Service.html#associatedtype.Future\" title=\"type tower_service::Service::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,\n    S: <a class=\"trait\" href=\"tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"axum/extract/type.Request.html\" title=\"type axum::extract::Request\">Request</a>, Response = <a class=\"type\" href=\"axum/response/type.Response.html\" title=\"type axum::response::Response\">Response</a>, Error = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.74.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,\n    S::<a class=\"associatedtype\" href=\"tower_service/trait.Service.html#associatedtype.Future\" title=\"type tower_service::Service::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.74.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,</span>"]],
"wasmtime_environ":[]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()