
# Experiments with [Hickory DNS]()

## To Investigate

Is it possible to create a resolver type that collects and handles failures so a set resolver can send a wide spread and log failures (or capture to metrics)?
* It is relatively straightforward to capture the error from a resolution that fails for a combined resolver.  
* It is still a little unclear how difficult capturing it would be to capture and record errors per request.

If none of the concurrent requests succeed does it move on and try again using the others in the set?
* is there an option for this type of behavior or would I have to implement this myself?
* Turns out it is easy to implement this yourself.

Why are rustls and openssl implementations broken?
* is it me or is it the library?
  * it seems like the library -- I tested on a separate computer with a different linux distro and received the same failures

## Variants

### DNS-over-TLS (DoT)

Supported via the `rustls`, `native-tls`, `openssl` backends. Currently only the
`native-tls` implementation works. 

The other two implementations are broken for the presets. 

### DNS-over-HTTPS (DoH)

`hickory` only supports this using `rustls` --- and it doesn't work. It fails with errors
for cloudflare, google, and quad9 for crate versions `v0.25.0-alpha.4` and `v0.24.2` the latest
and stable releases respectively. These all give the same errors on failure:

```txt
 ResolveError { kind: Proto(ProtoError { kind: Io(Custom { kind: InvalidData, error: InvalidCertificate(UnknownIssuer) }) }) }

 or

 ResolveError { kind: Proto(ProtoError { kind: Io(Os { code: 101, kind: NetworkUnreachable, message: "Network is unreachable" }) }) }
```


### DNS-over-HTTP3 (DoH3)

The only server currently supported in the configurations provided by `hickory` is `google_h3()` and
it is not currently working.

```rs
// requires that the `dns_over_h3` feature is enabled in `Cargo.toml`

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
```

fails with error

```txt
ResolveError { kind: Proto(ProtoError { kind: Io(Custom { kind: Other, error: TransportError(Error { code: Code::crypto(30), frame: None, reason: "invalid peer certificate: UnknownIssuer" }) }) }) }
```


### DNS-over-Quic (DoQ)

Currently there are no built-in hickory configurations that support DoQ.

There is a way to enter a custom DoQ configuration that would allow us to test / support this if we
find an endpoint that supports DoQ.

## Configuring options

```
ResolverConfig::ResolverOpts::server_ordering_strategy: ServerOrderingStrategy
	The server ordering strategy that the resolver should use.


ResolverConfig::ResolverOpts::num_concurrent_reqs: usize
	Number of concurrent requests per query
	
	Where more than one nameserver is configured, this configures the resolver to send queries to a number of servers in parallel. Defaults to 2; 0 or 1 will execute requests serially.
```
