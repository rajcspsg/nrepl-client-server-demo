# nrepl-client-demo

This is implementation of client of nrepl, a clojure based repl application.

## Running nrepl server
  The nrepl server is started using the below command and it requires clojure installed on system.

  ```bash
  clj -Sdeps '{:deps {nrepl/nrepl {:mvn/version "1.3.1"}}}' -m nrepl.cmdline
  ```

  Please follow these instructions to start nrepl server.

  ```bash
    cargo run -- server
  ```


## Running the client

To run the client provide the port number from running server.

  ```bash
      cargo run -- client 55419
  ```

Feel free to checkout and provide feedback.
