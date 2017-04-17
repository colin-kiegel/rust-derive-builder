# Vision
`derive_builder` provides an ergonomic and [idiomatic] way to deal with construction of guaranteed-complete objects from a range of inputs:

* Statically-known values
* Function arguments
* Wire/file/database inputs

A builder should produce only objects that uphold the invariants of the target type. This includes, but is not limited to, checks upheld by the type system.

## Documentation
Authors should focus documentation efforts on the target type.
* If the builder is the only way to create the target type, then the target type should discuss the requirements for its own creation.
* The builder can - in most cases - automatically include a link to the target type doc HTML from its own struct-level doc comment.

## Imports
A crate/module should always export the target type of a builder to allow the _built_ type to appear in function and struct type declarations. The exporting crate should _generally_ also expose the builder type, but may choose to exclude it from a prelude or the crate root, preferring to expose it in a child module. The crate is also free to keep the builder for its own internal use.

For the case of builder population with statically-known values or through explicit construction from function arguments, the target type should (optionally) expose a static `builder()` method which returns the builder type. In addition to keeping the imports shorter, this will appear in RLS ["find all reference"] queries by type, will be updated automatically during type renaming within the workspace observed by the language server, and will [show docs for the target type on type hover] (requesting docs on the static method will get the user to the builder-specific documentation). For more information on this, see the [language server protocol] that RLS is implementing.

## Fallible vs. Infallible Builders
The typesafe builder case (which is capable of statically validating that all required fields are present) can be achieved using a generic which marks when all required fields are present. A constructor on the builder can take non-optional values for all the required fields, and return a builder with the correct session type. By exposing inherent and trait `impl` blocks only on the generic instantiation with the correct state as its type parameter, the same implementation can serve both purposes.

## Validation
The builder should be focused specifically on the assembly of inputs and validating the completeness of the parameter set; additional validation should be handled by a constructor function provided by the target type. That constructor function may be public or private, depending on whether or not the crate author wants to _require_ the use of the builder.

# Use Case
Consider a `ConnectionPool` object. It has:

* A set of required fields, such as `host`, `api_key`, etc.
* A set of optional/defaulted fields: `timeout`, `retry_policy`, etc.
* A private TCP connection that is opened in `ConnectionPool::new` and closed during drop

This connection pool object's data is pulled from multiple sources, in decreasing priority:

1. Command-line args
1. Environment variables
1. A TOML config file

The union of the 3 sources must provide all the required values, or the `ConnectionPool` cannot be created. No individual source is required or expected to be complete.

Under this vision proposal, `ConnectionPool` would be declared as follows:

```rust

#[derive(Debug, Builder, Serialize)]
#[builder(name = "ConnectionInfo", 
    setter(into), 
    try_setter, 
    build_fn(skip), 
    derive(Deserialize), 
    preserve_attrs(serde))]
pub struct ConnectionPool {
    host: String,
    
    api_key: ApiKey,
    
    #[builder(default="ConnectionPool::default_timeout()")]
    timeout: Duration,
    
    #[builder(setter(skip))]
    #[serde(skip_serializing)]
    socket: Socket
}

impl ConnectionPool {
    /// Could be private if the builder was meant to be only way to create instance.
    pub fn new(host: String, api_key: ApiKey, timeout: Duration) -> Result<Self> {
        // validation would occur here
        
        let socket = unimplemented!(); // try to open the socket...
        ConnectionPool {
            host: host,
            api_key: api_key,
            timeout: timeout,
            socket: socket,
        }
    }
    
    fn default_timeout() -> Duration {
        Duration::from_millis(5000)
    }
}

impl ConnectionInfo {
    /// Establish a connection using the given information.
    pub fn connect(&self) -> Result<ConnectionPool> {
        ConnectionPool::new(
            self.get_host()?,
            self.get_api_key()?,
            self.get_timeout()?)
        )
    }
}
```

# Example
```rust
mod exporting_crate {
    use std::net::IpAddr;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Cycle {
        Auto,
        Slow,
        Medium,
        Fast,
    }

    impl FromStr for Cycle {
        type Err = String;
        // ... impl elided
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Key {
        Str(String),
        IpAddr(IpAddr),
    }

    #[derive(Debug, Clone)]
    pub enum Filter {
        Approx(String),
        Exact(Key),
    }

    impl Serialize for Filter {
        // ... impl elided
    }

    impl Deserialize for Filter {
        // ... impl elided
    }

    /// A data request for the Northwind metrics API.
    /// This type is focused on modeling a well-formed stat
    /// request, it doesn't allow for partially-built states.
    ///
    /// This invents a few new properties for the builder macro:
    /// 
    /// * `build_fn(skip)`: This tells the builder not to auto-generate the 
    ///   build method.
    /// * `preserve_attributes`: This tells the builder to include all #[serde] 
    ///   attributes on the `StatRequestBuilder` fields.
    /// * `target_factory`: Name needs work, but this tells the macro to add an 
    ///   inherent `builder` function on the target type.
    #[derive(Debug, Clone, Builder, Serialize)]
    #[builder(setter(into), try_setter, preserve_attrs(serde), derive(Deserialize), build_fn(skip), target_factory)]
    pub struct StatRequest {
        /// This particular field should be named `stat` on the inbound request;
        /// that requires a field-level annotation.
        #[serde(rename(deserialize = "stat"))]
        stat_name: String,

        cycle: Cycle,

        #[serde(default, rename = "filter", skip_serializing_if="Option::is_none")]
        #[builder(default)]
        key_filter: Option<Filter>,
    }

    impl StatRequest {
        pub fn new(stat_name: String, cycle: Cycle) -> Result<Self, String> {
            StatRequest::with_filter(stat_name, cycle, None)
        }

        /// Creates a new `StatRequest` with the specified filter (`None` is allowed).
        ///
        /// A filter is required for detail stats for perf reasons; this method will 
        /// return an error if one is needed and `None` is provided, or if one is provided
        /// and the specified stat does not support filtering.
        pub fn with_filter<F: Into<Option<Filter>>>(stat_name: String,
                                                   cycle: Cycle,
                                                   filter: F)
                                                   -> Result<Self, String> {
            let requires_filter = stat_name.ends_with("_detail");
            let filter_opt = filter.into();
            match (requires_filter, filter_opt.is_some()) {
                (true, false) => Err(format!("Stat `{}` requires a filter", stat_name)),
                (false, true) => Err(format!("Stat `{}` is not a detail metric")),
                _ => {
                    Ok(StatRequest {
                        stat_name: stat_name,
                        key_filter: filter_opt,
                        cycle: cycle,
                    })
                }
            }
        }
    }
    
    impl StatRequestBuilder {
        /// Create a stat request by calling `StatRequest::with_filter`. This method
        /// will return an error if a required field is not initialized, or if the 
        /// values fail validation.
        pub fn build(&self) -> Result<Self, String> {
            // These getters are private methods which return Result<T, String>;
            // the error arises if the field is uninitialized and has no default.
            StatRequest::with_filter(
                self.get_stat_name()?, 
                self.get_cycle()?, 
                self.get_filter()?)
        }
    }
    
    impl TryFrom<StatRequestBuilder> for StatRequest {
        type Error = String;
        
        fn try_from(v: StatRequestBuilder) -> Result<Self, Self::Error> {
            v.build()
        }
    }
}

mod consuming_crate {
    mod errors {
        error_chain! {
            // ... errors elided
        }
    }

    use errors::{Error, ChainedError, Result, ResultExt};
    use exporting_crate::{Cycle, Key, Filter, StatRequest};

    fn merge_args(inbound_req: &str, cycle_lock: Option<Cycle>) -> Result<StatRequest> {
        // deserialize from the inbound request into a builder; this enables further
        // validation and can be used to generate nicer failures if required fields are
        // missing. This is using error_chain and the carrier operator's conversion
        // feature to be very brief while still returning a strongly-typed error specific
        // to the consuming crate.
        let mut builder = serde_json::from_str(req_body)?;

        // apply the cycle override, if one was provided.
        if let Some(cl) = cycle_lock {
            builder.cycle(cl);
        }

        builder.build()
    }

    fn request(query: &StatRequest) -> Result<()> {
        let rsp = reqwest::Client::new()?
            .post("https://northwind.com/api/v1/metrics")
            .json(&query)
            .send()?;

        rsp.validate_status()
            .map(|| ())
            .chain_err(|| "The server returned an error")
    }

    /// Run an arbitrary stat request.
    pub fn run(req: &str) -> Result<()> {
        // Look for an environment-defined cycle lock and parse it if found.
        let cycle_lock = dotenv::var("CYCLE_LOCK").map(Cycle::from_str)?;

        // create the StatRequest; still relying on the error_chain + carrier pair
        // seen above. Now, `def` is a `StatRequest` which is ready to use.
        let query = merge_args(req, cycle_lock)?;
        
        request(query)
    }
    
    /// Test if the service is up or down.
    pub fn health_check(cycle: Cycle) -> Result<()> {
        // this gets a fresh StatRequestBuilder, despite not having
        // imported the
        let query = StatRequest::builder()
            .stat_name("system.is_up")
            .cycle(cycle)
            .build()
            .chain_err(|| "Health check is malformed")?;
            
        request(query);
    }
}
```

[idiomatic]: https://aturon.github.io/ownership/builders.html
["find all references"]: https://github.com/rust-lang-nursery/rls/blob/master/src/server.rs#L162
[show docs for the target type on type hover]: https://github.com/rust-lang-nursery/rls/blob/master/src/server.rs#L160
[language server protocol]: https://github.com/Microsoft/language-server-protocol