
pub fn select_free_port() -> u16 {
    let socket = std::net::TcpListener::bind("localhost:0").unwrap(); // Port 0 requests a free port from the operating system
    socket.local_addr().unwrap().port()
    //socket is dropped at the end of this method, which releases the bound port, allowing us to use it
}
