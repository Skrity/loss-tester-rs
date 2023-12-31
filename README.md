# Network loss tester

## Usage

```text
Program to detect network packet loss and packet mangling

Usage: loss-tester-rs.exe [OPTIONS] <COMMAND>

Commands:
  server  Server mode
  client  Client mode
  help    Print this message or the help of the given subcommand(s)

Options:
  -P, --proto <PROTO>  Protocol to send data over [default: udp] [possible values: udp, tcp]
  -B, --bind <BIND>    IP address to bind to [default: 0.0.0.0]
  -h, --help           Print help
  -V, --version        Print version

```

### Server mode

```text
Server mode

Usage: loss-tester-rs.exe server [OPTIONS] <ADDR> [PORT]

Arguments:
  <ADDR>  IP address to serve on
  [PORT]  Port to serve on [default: 5000]

Options:
  -I, --interval <INTERVAL>  Interval between reports [default: 1]
  -h, --help                 Print help

```

### Client mode

```text
Client mode

Usage: loss-tester-rs.exe client [OPTIONS] <ADDR> [PORT]

Arguments:
  <ADDR>  IP address to connect to
  [PORT]  Port to connect to [default: 5000]

Options:
  -b, --bandwidth <BANDWIDTH>  Limit transmission bandwidth, kbit/s (0 to disable limiting) [default: 1000]
  -m, --mtu <MTU>              Maximum Transmission Unit [default: 1500]
  -h, --help                   Print help

```
