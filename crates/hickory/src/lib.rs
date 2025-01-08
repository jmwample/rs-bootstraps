//! Client API tools and implementations for DNS resolution while using the API client.
//!

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use hickory_resolver::{
    config::{LookupIpStrategy, ResolverConfig, ResolverOpts},
    lookup_ip::LookupIpIntoIter,
    ResolveError, TokioResolver,
};
use once_cell::sync::OnceCell;

/// Wrapper around an `AsyncResolver`, which implements the `Resolve` trait.
#[derive(Debug, Default, Clone)]
pub struct HickoryDnsResolver {
    /// Since we might not have been called in the context of a
    /// Tokio Runtime in initialization, so we must delay the actual
    /// construction of the resolver.
    state: Arc<OnceCell<TokioResolver>>,
    fallback: Arc<OnceCell<TokioResolver>>,
}

struct SocketAddrs {
    iter: LookupIpIntoIter,
}

#[derive(Debug)]
struct HickoryDnsSystemConfError(ResolveError);

impl Resolve for HickoryDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.state.clone();
        let fallback = self.fallback.clone();
        Box::pin(async move {
            let resolver = resolver.get_or_try_init(new_resolver)?;

            // try the primary DNS resolver that we set up (DoH or DoT or whatever)
            let lookup = match resolver.lookup_ip(name.as_str()).await {
                Ok(res) => res,
                Err(e) => {
                    // on failure use the fall back system configured DNS resolver
                    println!("primary DNS failed w/ error {e}: using system fallback");
                    let resolver = fallback.get_or_try_init(new_resolver_system)?;
                    resolver.lookup_ip(name.as_str()).await?
                }
            };

            let addrs: Addrs = Box::new(SocketAddrs {
                iter: lookup.into_iter(),
            });
            Ok(addrs)
        })
    }
}

/// Wrapper around an `AsyncResolver`, which implements the `Resolve` trait.
#[derive(Debug, Default, Clone)]
pub struct HickoryDnsResolver1 {
    state: Arc<OnceCell<TokioResolver>>,
}

impl Resolve for HickoryDnsResolver1 {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.state.clone();
        Box::pin(async move {
            let resolver = resolver.get_or_try_init(new_resolver)?;

            let lookup = resolver.lookup_ip(name.as_str()).await?;

            let addrs: Addrs = Box::new(SocketAddrs {
                iter: lookup.into_iter(),
            });
            Ok(addrs)
        })
    }
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ip_addr| SocketAddr::new(ip_addr, 0))
    }
}

/// Create a new resolver with the default configuration,
/// which reads from `/etc/resolve.conf`. The options are
/// overridden to look up for both IPv4 and IPv6 addresses
/// to work with "happy eyeballs" algorithm.
fn new_resolver() -> Result<TokioResolver, HickoryDnsSystemConfError> {
    // let (config, mut opts) = hickory_resolver::system_conf::read_system_conf().map_err(HickoryDnsSystemConfError)?;
    let config = ResolverConfig::google_tls();
    let mut opts = ResolverOpts::default();
    opts.ip_strategy = LookupIpStrategy::Ipv4thenIpv6;
    Ok(TokioResolver::tokio(config, opts))
}

/// Create a new resolver with the default configuration,
/// which reads from `/etc/resolve.conf`. The options are
/// overridden to look up for both IPv4 and IPv6 addresses
/// to work with "happy eyeballs" algorithm.
fn new_resolver_system() -> Result<TokioResolver, HickoryDnsSystemConfError> {
    let (config, mut opts) =
        hickory_resolver::system_conf::read_system_conf().map_err(HickoryDnsSystemConfError)?;
    opts.ip_strategy = LookupIpStrategy::Ipv4thenIpv6;
    Ok(TokioResolver::tokio(config, opts))
}

impl fmt::Display for HickoryDnsSystemConfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("error reading DNS system conf for hickory-dns")
    }
}

impl std::error::Error for HickoryDnsSystemConfError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Try instantiating reqwest in a way that uses our resolver with DoH enabled
    ///
    /// Plain DNS and DNS-over-TLS works for `v0.24.2` and `v0.25.0-alpha.4`
    ///
    /// ```rs
    /// fn new_resolver() -> Result<TokioAsyncResolver, HickoryDnsSystemConfError> {
    ///     let config = ResolverConfig::google_tls();
    ///     let mut opts = ResolverOpts::default();
    ///     //...
    /// }
    /// ```
    ///
    #[tokio::test]
    async fn reqwest_hickory_doh_fallback() {
        let resolver = HickoryDnsResolver::default();
        let client = reqwest::ClientBuilder::new()
            .dns_resolver(resolver.into())
            .build()
            .unwrap();

        let resp = client
            .get("http://httpbin.org/ip")
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        println!("bytes: {resp:?}");
    }


    #[tokio::test]
    async fn reqwest_hickory_no_fallback() {
        let resolver = HickoryDnsResolver1::default();
        let client = reqwest::ClientBuilder::new()
            .dns_resolver(resolver.into())
            .build()
            .unwrap();

        let resp = client
            .get("http://httpbin.org/ip")
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        println!("bytes: {resp:?}");
    }
}

#[cfg(test)]
mod tests_v0_25_0;

#[cfg(test)]
#[cfg(feature = "disabled")]
mod tests_v0_24_2;
