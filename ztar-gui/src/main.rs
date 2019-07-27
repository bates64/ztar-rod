use std::path::Path;

use iron::Iron;
use mount::Mount;
use staticfile::Static;

fn main() {
    let mut mount = Mount::new();

    // Serve static directory.
    mount.mount("/", Static::new(Path::new("static/")));

    // Pick an open port, start the webserver, and open it in a web browser.
    let port = porthole::open().expect("no open ports on localhost!");
    println!("running on port {}", port);

    let _ = open::that(format!("http://localhost:{}/", port)); // Ignore failure.

    Iron::new(mount)
        .http(format!("127.0.0.1:{}", port))
        .unwrap();
}
