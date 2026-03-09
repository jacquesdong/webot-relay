import argparse
import os
from http.server import BaseHTTPRequestHandler, HTTPServer
import urllib.request
from typing import Dict, Optional, Union, Tuple

class RelayHTTPRequestHandler(BaseHTTPRequestHandler):
    webot_url: Optional[str] = None
    
    def do_POST(self) -> None:
        # Read request body
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length)
        
        # Get request headers
        request_headers = dict(self.headers)
        
        if self.webot_url:
            # Relay request to WEBOT_URL
            try:
                req = urllib.request.Request(
                    self.webot_url,
                    data=post_data,
                    headers=request_headers,
                    method='POST'
                )
                with urllib.request.urlopen(req) as response:
                    response_data = response.read()
                    response_status = response.getcode()
                
                # Send response back to client
                self.send_response(response_status)
                for key, value in response.getheaders():
                    self.send_header(key, value)
                self.end_headers()
                self.wfile.write(response_data)
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(f"Error relaying request: {str(e)}".encode('utf-8'))
        else:
            # Print request headers and body
            print("Request Headers:")
            for key, value in request_headers.items():
                print(f"{key}: {value}")
            print("\nRequest Body:")
            try:
                print(post_data.decode('utf-8'))
            except UnicodeDecodeError:
                print(f"Binary data: {len(post_data)} bytes")
            
            # Send response
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b"Request received and printed")

def parse_address(addr: str) -> Tuple[str, int]:
    """Parse address string into host and port"""
    if addr.startswith(':'):
        host = ''
        port = int(addr[1:])
    else:
        parts = addr.split(':')
        host = parts[0]
        port = int(parts[1]) if len(parts) > 1 else 8001
    return host, port

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("addr", default=":8001", help="Address to bind to (e.g., :8001 or localhost:8001)")
    
    args = parser.parse_args()
    
    # Get WEBOT_URL from environment
    webot_url = os.environ.get("WEBOT_URL", "http://localhost:8002")
    
    # Set the webot_url in the handler class
    RelayHTTPRequestHandler.webot_url = webot_url if webot_url else None
    
    # Parse address
    host, port = parse_address(args.addr)
    
    # Create and start server
    server = HTTPServer((host, port), RelayHTTPRequestHandler)
    print(f"Server running on {host}:{port}")
    print(f"WEBOT_URL: {webot_url}")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped.")

if __name__ == "__main__":
    main()
