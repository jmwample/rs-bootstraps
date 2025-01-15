
# Experiments with [Hickory DNS]()

## To Investigate

Is it possible to create a resolver type that collects and handles failures so a set resolver can send a wide spread and log failures (or capture to metrics)?
* It is relatively straightforward to capture the error from a resolution that fails for a combined resolver.  
* It is still a little unclear how difficult capturing it would be to capture and record errors per request.

If none of the concurrent requests succeed does it move on and try again using the others in the set?
* is there an option for this type of behavior or would I have to implement this myself?
* Answer: Turns out it is easy to implement this yourself.

Why are rustls and openssl implementations broken?
* is it me or is it the library?
  * ~~it seems like the library. I tested on a separate computer with a different linux distro and received the same failures~~
  * It was me (kind of) the hickory library [doesn't come with any default https
    roots](https://github.com/hickory-dns/hickory-dns/issues/2066) so you have to add a feature
    otherwise https just fails every time with an `UnknownIssuer` error.

## Variants

### DNS-over-TLS (DoT)

Supported via the `rustls`, `native-tls`, `openssl` backends. The
`native-tls` implementation works out of the box.

The other two implementations are broken for the presets. THAT IS, they are broken until you enable one of the features that tells hickory what roots to use (`webpki-roots` or `native-certs`).

### DNS-over-HTTPS (DoH)

`hickory` only supports this using `rustls`.

### DNS-over-HTTP3 (DoH3)

The only server currently supported in the configurations provided by `hickory` is `google_h3()`. Requires `dns-over-h3` and one of the pki roots feature flags.

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
