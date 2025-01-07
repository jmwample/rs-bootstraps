//! Client API tools and implementations for DNS resolution while using the API client.
//!

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use hickory_resolver::{
    config::{LookupIpStrategy, ResolverConfig, ResolverOpts},
    ResolveError,
    lookup_ip::LookupIpIntoIter,
    TokioResolver,
};
use once_cell::sync::OnceCell;

/// Wrapper around an `AsyncResolver`, which implements the `Resolve` trait.
#[derive(Debug, Default, Clone)]
pub struct HickoryDnsResolver {
    /// Since we might not have been called in the context of a
    /// Tokio Runtime in initialization, so we must delay the actual
    /// construction of the resolver.
    state: Arc<OnceCell<TokioResolver>>,
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
fn new_resolver() -> Result<TokioResolver, HickoryDnsSystemConfError> {
    // let (config, mut opts) = hickory_resolver::system_conf::read_system_conf().map_err(HickoryDnsSystemConfError)?;
    let config = ResolverConfig::google_tls();
    let mut opts = ResolverOpts::default();
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


/// # To look into
/// 
/// ResolverConfig::ResolverOpts::server_ordering_strategy -> ServerOrderingStrategy
/// ```
///     server_ordering_strategy: ServerOrderingStrategy
///         The server ordering strategy that the resolver should use.
/// ```
///  
/// ResolverConfig::ResolverOpts::num_concurrent_reqs -> usize
/// ```
///     num_concurrent_reqs: usize
///         Number of concurrent requests per query
///         
///         Where more than one nameserver is configured, this configures the resolver to send queries to a number of servers in parallel. Defaults to 2; 0 or 1 will execute requests serially.
/// ```
/// 
/// ---
/// 
/// Is it possible to create a resolver type that collect and handles failures so a set resolver can send a wide spread and log failures (or capture to metrics)?
/// 
/// If none of the concurrent requests succeed does it move on and try again using the others in the set?
/// 
/// Why are rustls and openssl implementations broken?
/// * is it me or is it the library?

#[cfg(test)]
mod tests {
    use super::*;

    use hickory_resolver::config::NameServerConfigGroup;

    use std::net::*;

    use hickory_resolver::Resolver;


    /// Attempt to use a set of resolvers using DNS-over-___ protocols
    /// 
    /// Performs a DNS lookup using a custom hickory-dns resolver. The resolver
    /// itself is the set combination of google, cloudflare, and quad9 combining DNS-over-TLS
    /// and traditional DNS over UDP. This test fails when the rustls implementation is used
    /// which means that we can't use DoH -- but in theory it could be include transparently.
    /// This test requires that the `hickory_resolver` crates has the `dns-over-native-tls`
    /// feature enabled.
    /// 
    /// The DNS-over-HTTPS implementation is also broken for `hickory_resolver@v0.25.0-alpha.4`.
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn dns_over_tls_NameServerConfigGroup_set() {
        let mut name_servers = NameServerConfigGroup::google();
        // name_servers.merge(NameServerConfigGroup::cloudflare_https());
        name_servers.merge(NameServerConfigGroup::quad9_tls());

        let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);

        // Construct a new Resolver with default configuration options
        let resolver =
            Resolver::tokio(config, ResolverOpts::default());

        // Lookup the IP addresses associated with a name.
        let response = resolver.lookup_ip("www.example.com.").await.unwrap();

        // There can be many addresses associated with the name,
        //  this can return IPv4 and/or IPv6 addresses
        let address = response.iter().next().expect("no addresses returned!");
        if address.is_ipv4() {
            assert_eq!(address, IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14)));
        } else {
            assert_eq!(
                address,
                IpAddr::V6(Ipv6Addr::new(
                    0x2606, 0x2800, 0x21f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c
                ))
            );
        }
    }

    /// Attempt to use a set of resolvers using DNS-over-TLS
    /// 
    /// Performs a DNS lookup using a custom hickory-dns resolver. The resolver
    /// itself is the set combination of the google, cloudflare, and quad9 DNS-over-TLS
    /// services. This requires that the `hickory_resolver` crates has the `dns-over-native-tls`
    /// feature enabled.
    /// 
    /// ```
    /// hickory-resolver = {version="0.24.2", features=[ "dns-over-native-tls"]}
    /// ```
    #[tokio::test]
    #[allow(non_snake_case)]
    async fn dns_over_any_NameServerConfigGroup_set() {
        let mut name_servers = NameServerConfigGroup::google_tls();
        name_servers.merge(NameServerConfigGroup::cloudflare_tls());
        name_servers.merge(NameServerConfigGroup::quad9_tls());

        let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);

        // Construct a new Resolver with default configuration options
        let resolver =
            Resolver::tokio(config, ResolverOpts::default());

        // Lookup the IP addresses associated with a name.
        let response = resolver.lookup_ip("www.example.com.").await.unwrap();

        // There can be many addresses associated with the name,
        //  this can return IPv4 and/or IPv6 addresses
        let address = response.iter().next().expect("no addresses returned!");
        if address.is_ipv4() {
            assert_eq!(address, IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14)));
        } else {
            assert_eq!(
                address,
                IpAddr::V6(Ipv6Addr::new(
                    0x2606, 0x2800, 0x21f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c
                ))
            );
        }
    }

    /// Try instantiating reqwest in a way that uses our resolver with DoH enabled
    /// 
    /// Plain DNS and DNS-over-TLS work for `hickory_resolver@v0.24.2`
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

#[cfg(test)]
#[cfg(feature="disabled")]
mod tests_hickory_resolver_0_24_2 {
    use super::*;


    /// Try instantiating reqwest in a way that uses our resolver with DoH enabled
    /// 
    /// Plain DNS and DNS-over-TLS work for `hickory_resolver@v0.24.2`
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

    /*

    use hickory_resolver::config::NameServerConfigGroup;

    use std::net::*;

    use hickory_resolver::Resolver;

    /// Attempt to use a set of resolvers using DNS-over-TLS
    /// 
    /// Performs a DNS lookup using a custom hickory-dns resolver. The resolver
    /// itself is the set combination of the google, cloudflare, and quad9 DNS-over-TLS
    /// services. This requires that the `hickory_resolver` crates has the `dns-over-native-tls`
    /// feature enabled (Note: the rustls and openssl implementations are broken for
    /// `hickory_resolver@v0.24.2`).
    /// 
    /// ```
    /// hickory-resolver = {version="0.24.2", features=[ "dns-over-native-tls"]}
    /// ```
    /// 
    /// The DNS-over-HTTPS implementation is also broken for `hickory_resolver@v0.24.2`.
    /// 
    /// ---
    /// 
    /// Questions:
    /// How are they selected from the set?
    /// * Does it try all of them in parallel?
    /// * Does it pick one at random?
    /// * Are they tried in the order that they are added?
    /// If one fails does it fall back to the others in the set?
    #[test]
    #[allow(non_snake_case)]
    fn dns_over_tls_NameServerConfigGroup_set() {
        let mut name_servers = NameServerConfigGroup::google_tls();
        name_servers.merge(NameServerConfigGroup::cloudflare_tls());
        name_servers.merge(NameServerConfigGroup::quad9_tls());

        let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);

        // Construct a new Resolver with default configuration options
        let resolver =
            Resolver::new(config, ResolverOpts::default()).unwrap();

        // Lookup the IP addresses associated with a name.
        let response = resolver.lookup_ip("www.example.com.").unwrap();

        // There can be many addresses associated with the name,
        //  this can return IPv4 and/or IPv6 addresses
        let address = response.iter().next().expect("no addresses returned!");
        if address.is_ipv4() {
            assert_eq!(address, IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14)));
        } else {
            assert_eq!(
                address,
                IpAddr::V6(Ipv6Addr::new(
                    0x2606, 0x2800, 0x21f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c
                ))
            );
        }
    }
    */
}
