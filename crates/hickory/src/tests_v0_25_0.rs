
use super::*;

use hickory_resolver::config::NameServerConfigGroup;

use std::net::*;

use hickory_resolver::Resolver;


use std::env;
use std::sync::Once;
use std::str::FromStr;

use tracing_subscriber::filter::LevelFilter;

static SUBSCRIBER_INIT: Once = Once::new();

pub fn init_subscriber() {
	SUBSCRIBER_INIT.call_once(|| {
		let level = env::var("RUST_LOG_LEVEL").unwrap_or("error".into());
		let lf = LevelFilter::from_str(&level).unwrap();

		tracing_subscriber::fmt().with_max_level(lf).init();
	});
}    

#[tokio::test]
async fn dns_over_https() {
	init_subscriber();

	// Construct a new Resolver with default configuration options
	let resolver = Resolver::tokio(ResolverConfig::quad9_https(), ResolverOpts::default());

	// Lookup the IP addresses associated with a name.
	let response = resolver.lookup_ip("www.example.com.").await.unwrap();

	// There can be many addresses associated with the name,
	//  this can return IPv4 and/or IPv6 addresses
	let address = response.iter().next().expect("no addresses returned!");
	println!("{ }", address);
	let expected = [
		IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14)),
		IpAddr::V4(Ipv4Addr::new(23, 201, 195, 209)),
		IpAddr::V6(Ipv6Addr::new(
			0x2606, 0x2800, 0x21f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c,
		)),
	];
	assert!(expected.contains(&address));
}

#[tokio::test]
async fn dns_over_h3() {
	// Construct a new Resolver with default configuration options
	let resolver = Resolver::tokio(ResolverConfig::google_h3(), ResolverOpts::default());

	// Lookup the IP addresses associated with a name.
	let response = resolver.lookup_ip("www.example.com.").await.unwrap();

	// There can be many addresses associated with the name,
	//  this can return IPv4 and/or IPv6 addresses
	let address = response.iter().next().expect("no addresses returned!");
	let expected = [
		IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14)),
		IpAddr::V6(Ipv6Addr::new(
			0x2606, 0x2800, 0x21f, 0xcb07, 0x6820, 0x80da, 0xaf6b, 0x8b2c,
		)),
	];
	assert!(expected.contains(&address));
}

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
	let resolver = Resolver::tokio(config, ResolverOpts::default());

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
	let resolver = Resolver::tokio(config, ResolverOpts::default());

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