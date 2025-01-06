//! Client API tools and implementations for DNS resolution while using the API client.
//!

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use hickory_resolver::{
    config::LookupIpStrategy,
    config::{ResolverConfig, ResolverOpts},
    error::ResolveError,
    lookup_ip::LookupIpIntoIter,
    TokioAsyncResolver,
};
use once_cell::sync::OnceCell;

/// Wrapper around an `AsyncResolver`, which implements the `Resolve` trait.
#[derive(Debug, Default, Clone)]
pub struct HickoryDnsResolver {
    /// Since we might not have been called in the context of a
    /// Tokio Runtime in initialization, so we must delay the actual
    /// construction of the resolver.
    state: Arc<OnceCell<TokioAsyncResolver>>,
}

struct SocketAddrs {
    iter: LookupIpIntoIter,
}

#[derive(Debug)]
struct HickoryDnsSystemConfError(ResolveError);

impl Resolve for HickoryDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.clone();
        Box::pin(async move {
            let resolver = resolver.state.get_or_try_init(new_resolver)?;

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
fn new_resolver() -> Result<TokioAsyncResolver, HickoryDnsSystemConfError> {
    // let (config, mut opts) = hickory_resolver::system_conf::read_system_conf().map_err(HickoryDnsSystemConfError)?;
    let config = ResolverConfig::quad9_tls();
    let mut opts = ResolverOpts::default();
    opts.ip_strategy = LookupIpStrategy::Ipv4thenIpv6;
    Ok(TokioAsyncResolver::tokio(config, opts))
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

    // use std::net::*;

    // use hickory_resolver::Resolver;

    // /// Attempt to use the Resolver directly
    // #[test]
    // fn it_works() {
    //     // Construct a new Resolver with default configuration options
    //     let resolver =
    //         Resolver::new(ResolverConfig::cloudflare_https(), ResolverOpts::default()).unwrap();

    //     // Lookup the IP addresses associated with a name.
    //     let response = resolver.lookup_ip("www.example.com.").unwrap();

    //     // There can be many addresses associated with the name,
    //     //  this can return IPv4 and/or IPv6 addresses
    //     let address = response.iter().next().expect("no addresses returned!");
    //     if address.is_ipv4() {
    //         assert_eq!(address, IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14)));
    //     } else {
    //         assert_eq!(
    //             address,
    //             IpAddr::V6(Ipv6Addr::new(
    //                 0x2606, 0x2800, 0x21f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c
    //             ))
    //         );
    //     }

    //     // let resolver = HickoryDnsResolver::default();
    //     // let addr = resolver
    //     //     .resolve(Name::from_str("unknown.wampler.co").unwrap())
    //     //     .await
    //     //     .expect("failed to do the thing");
    // }

    /// Try instantiating reqwest in a way that uses our resolver with DoH enabled
    #[tokio::test]
    async fn reqwest_hickory_doh() {
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
}
