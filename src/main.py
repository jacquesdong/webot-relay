import os
import sys

import argparse

from http.server import BaseHTTPRequestHandler, HTTPServer
import urllib.request

from typing import Tuple

class RelayHTTPRequestHandler(BaseHTTPRequestHandler):
    def do_POST(self) -> None:
        # Read request body
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length)
        
        # Get request headers
        request_headers = dict(self.headers)

        if args.verbose:
            # Print request headers and body
            print("Request Headers:", file=sys.stderr)
            for key, value in request_headers.items():
                print(f"{key}: {value}", file=sys.stderr)
            print("\nRequest Body:", file=sys.stderr)
            try:
                print(post_data.decode('utf-8'), file=sys.stderr)
            except UnicodeDecodeError:
                print(f"Binary data: {len(post_data)} bytes", file=sys.stderr)
                print(post_data, file=sys.stderr)

        if args.url:
            # Relay request to WEBOT_URL
            try:
                req = urllib.request.Request(
                    args.url,
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
            except Exception as err:
                self.send_response(500)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                error_response = '{"errcode": 1, "errmsg": "Error relaying request: ' + str(err) + '"}'
                self.wfile.write(error_response.encode('utf-8'))
        else:
            # Send response
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            success_response = '{"errcode": 0, "errmsg": "ok"}'
            self.wfile.write(success_response.encode('utf-8'))

def parse_address(addr: str) -> Tuple[str, int]:
    """Parse address string into host and port"""
    if addr.startswith(':'):
        host = ''
        port = int(addr[1:])
    else:
        parts = addr.split(':')
        host = parts[0]
        port = int(parts[1]) if len(parts) > 1 else 8000
    return host, port

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("addr", nargs='?', default=":8000", help="Address to bind to (e.g., :8000 or localhost:8000)")
    parser.add_argument("--url", help="URL to relay requests to")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose logging")

    global args
    args = parser.parse_args()

    # Parse address
    host, port = parse_address(args.addr)

    # Get WEBOT_URL from environment or use argument
    if not args.url:
        args.url = os.environ.get("WEBOT_URL")
    
    # Create and start server
    server = HTTPServer((host, port), RelayHTTPRequestHandler)
    
    if args.verbose:
        print(f"Server running on {host}:{port}", file=sys.stderr)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped.", file=sys.stderr)

if __name__ == "__main__":
    main()
