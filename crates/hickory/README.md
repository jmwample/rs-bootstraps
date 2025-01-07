
# Experiments with [Hickory DNS]()

## Variants

### DoH

hickory only supports this using `rustls` - it doesn't work. It fails with errors
for cloudflare, google, and quad9.

### DoT

Supported via the `rustls`, `native-tls`, `openssl` backends. Currently only the
`native-tls` implementation works. 

The other two implementations are broken for the presets. 

### DoH3

### DoQ

## Configuring options
