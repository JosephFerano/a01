## Consumer/Producer in Rust for CCOM4017

The code is organized into 3 files; `client.rs`, `server.rs`, and `lib.rs`. Rust automagically generates binaries for client and server, lib simply has a small amount of shared code.

#### Server

To run the server, run
```./server <IP> <PORT>```

#### Client

For the client, run
```./client <MOBILE_ID> <IP> <SERVER_PORT> <CLIENT_PORT>```

The client port is needed because multiple clients need to bind to their own port.

Run as many of clients as you want, so long as the ports are different.

#### Building

If you wish to compile the code, install rust and cargo
Link(https://www.rust-lang.org/en-US/install.html)

Then for the server just run;
```cargo run --bin server 127.0.0.1 7788```

For the client;
```cargo run --bin client 1556 127.0.0.1 7788 7780```


