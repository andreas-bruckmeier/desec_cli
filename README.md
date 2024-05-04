# deSEC Client

Commandline client for the [deSEC](https://desec.io/) DNS API.

deSEC is a free DNS hosting service, designed with security in mind.
Running on open-source software and supported by [SSE](https://securesystems.de/), deSEC is free for everyone to use.

## Examples

Usage dialog
```
Usage: desec_cli [OPTIONS] <COMMAND>

Commands:
  account  Manage account or create a new one
  domain   Manage domains
  rrset    Manage Resource Record Sets
  token    Manage Token
  policy   Manage Token Policies
  help     Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet                      Error messages are suppressed
      --no-retry                   Whether to disable retry of throttled requests which would incure sleeps
      --max-wait <MAX_WAIT>        Maximum time to wait between retries of throttled requests
      --max-retries <MAX_RETRIES>  Maximum number of retries per request
  -h, --help                       Print help
```

Create new domain
```
desec_cli domain create desec_cli.com | jq
{
  "created": "2024-05-04T16:24:28.660313Z",
  "keys": [
    {
      "dnskey": "...",
      "ds": [
        "...",
        "..."
      ],
      "flags": 257,
      "keytype": "csk",
      "managed": true
    }
  ],
  "minimum_ttl": 3600,
  "name": "desec_cli.com",
  "published": null,
  "touched": "2024-05-04T16:24:29.126251Z",
  "zonefile": null
}
```

List domains
```
desec_cli domain list | jq
[
  {
    "created": "2024-05-04T16:24:28.660313Z",
    "keys": null,
    "minimum_ttl": 3600,
    "name": "desec_cli.com",
    "published": "2024-05-04T16:24:29.155983Z",
    "touched": "2024-05-04T16:24:29.155983Z",
    "zonefile": null
  },
  ...
]
```

## License

See [LICENSE-MIT](LICENSE-MIT) for details.
