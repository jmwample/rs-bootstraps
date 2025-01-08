
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