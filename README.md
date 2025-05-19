# ret

A simple proxy tool that supports HTTP proxy, TCP forwarding, Shadowsocks, and tunnel proxy.


## Features
Supports configuration of an upstream HTTP proxy.

The client can function as an HTTP(S) or SOCKS5 proxy.

The server is compatible with the standard SOCKS5 protocol and also includes optional encryption negotiation.

Tunnels UDP over TCP: UDP packets are transmitted through a TCP channel to bypass firewall restrictions.

Supports multiple encryption methods (TLS, AES-256-CFB, DES-CFB, RC4-MD5, etc.).

The client is compatible with the Shadowsocks protocol and can be used as a Shadowsocks server.

## Usage
1.start an HTTP proxy service
```bash
$ ret -L=:443
```
2.http forwarding
```bash
$ ret -L http://:8080 -F http://192.168.1.1:8080
```
3.shadowsocks tunnel
```bash
$ ret -L http://:8080 -F ss://192.168.1.1:8081
```
4.TCP local port forwarding
```bash
$ ret -L tcp://:8080/192.168.1.1:80
```




