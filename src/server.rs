use regex::Regex;
use std::io;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// A server manager for launching and controlling an nREPL server process.
///
/// `NreplServer` provides methods to start an nREPL server using Clojure CLI or Leiningen,
/// check its status, retrieve the port, read output, and stop the server.
pub struct NreplServer {
    child: Option<Child>,
    port: Option<u16>,
}

impl NreplServer {
    /// Creates a new `NreplServer` instance with no running process.
    ///
    /// # Returns
    ///
    /// Returns a new `NreplServer` with no child process or port set.
    pub fn new() -> Self {
        Self {
            child: None,
            port: None,
        }
    }

    fn parse_port_from_output(&self, line: &str) -> Option<u16> {
        let re = Regex::new(r"port (\d+)").ok()?;
        let caps = re.captures(line)?;
        caps[1].parse::<u16>().ok()
    }

    /// Starts an nREPL server using the Clojure CLI (`clj`).
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the port number the server is listening on if successful,
    /// or an `io::Error` if the server fails to start.
    pub fn start_with_clj(&mut self) -> io::Result<u16> {
        let mut cmd = Command::new("clj");

        let args = [
            "-Sdeps",
            "{:deps {nrepl/nrepl {:mvn/version \"1.3.1\"}}}",
            "-M",
            "-m",
            "nrepl.cmdline",
        ];

        let mut child = cmd
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut confirmed_port = 0;
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines_iter = reader.lines();

            // Give it some time to start and read a few lines
            for _ in 0..10 {
                if let Some(Ok(line)) = lines_iter.next() {
                    if let Some(port) = self.parse_port_from_output(&line) {
                        confirmed_port = port;
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(200));
            }
        }

        self.child = Some(child);
        self.port = Some(confirmed_port);

        Ok(confirmed_port)
    }

    /// Starts an nREPL server using Leiningen (`lein repl :headless`).
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the port number the server is listening on if successful,
    /// or an `io::Error` if the server fails to start.
    pub fn start_with_lein(&mut self) -> io::Result<u16> {
        let mut cmd = Command::new("lein");
        cmd.args(&["repl", ":headless"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let mut confirmed_port = 0;
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines_iter = reader.lines();

            // Give it some time to start and read a few lines
            for _ in 0..10 {
                if let Some(Ok(line)) = lines_iter.next() {
                    if let Some(port) = self.parse_port_from_output(&line) {
                        confirmed_port = port;
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
        self.child = Some(child);
        self.port = Some(confirmed_port);

        // Give the server time to start
        thread::sleep(Duration::from_millis(2000));

        Ok(confirmed_port)
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,     // Process is still running
                Err(_) => false,      // Error checking status
            }
        } else {
            false
        }
    }

    /// Returns the port number the nREPL server is listening on, if known.
    ///
    /// # Returns
    ///
    /// Returns `Some(port)` if the server port is known, or `None` otherwise.
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Reads and collects output lines from the server process's stdout.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of output lines if successful,
    /// or an `io::Error` if reading fails.
    pub fn read_output(&mut self) -> io::Result<Vec<String>> {
        let mut lines = Vec::new();

        if let Some(ref mut child) = self.child {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(l) => lines.push(l),
                        Err(_) => break,
                    }
                }
            }
        }

        Ok(lines)
    }

    /// Stops the nREPL server process if it is running.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server was stopped successfully,
    /// or an `io::Error` if stopping the process fails.
    pub fn stop(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }
}

impl Drop for NreplServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let mut server = NreplServer::new();
        assert!(!&server.is_running());
        assert_eq!(server.port(), None);
    }

    #[test]
    fn test_server_lifecycle() {
        let mut server = NreplServer::new();

        assert!(!server.is_running());

        let result = server.stop();
        assert!(result.is_ok());
    }

    #[test]
    /* fn test_find_available_port() {
        let port = NreplServer::find_available_port();
        assert!(port.is_ok());
        let port_num = port.unwrap();
        assert!(port_num > 0);
    } */
    #[test]
    fn test_parse_port_from_output() {
        let server = NreplServer::new();

        assert_eq!(
            server.parse_port_from_output("nREPL server started on port 7888"),
            Some(7888)
        );
        assert_eq!(
            server.parse_port_from_output("Started nREPL on port 1234 at localhost"),
            Some(1234)
        );
        assert_eq!(server.parse_port_from_output("No port info here"), None);
    }
}
