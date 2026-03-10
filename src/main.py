import os
import sys
import argparse
from http.server import BaseHTTPRequestHandler, HTTPServer
import urllib.request
from typing import Tuple, Optional


class RelayHTTPRequestHandler(BaseHTTPRequestHandler):
    url = None
    verbose = False

    def do_POST(self) -> None:
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length)
        request_headers = dict(self.headers)

        if self.__class__.verbose:
            sys.stderr.write("Request Headers:\n")
            for key, value in request_headers.items():
                sys.stderr.write("{}: {}\n".format(key, value))
            sys.stderr.write("\nRequest Body:\n")
            try:
                sys.stderr.write(post_data.decode('utf-8'))
            except UnicodeDecodeError:
                sys.stderr.write("Binary data: {} bytes\n".format(len(post_data)))
                sys.stderr.write(str(post_data))

        if self.__class__.url:
            try:
                req = urllib.request.Request(
                    self.__class__.url,
                    data=post_data,
                    headers=request_headers,
                    method='POST'
                )
                with urllib.request.urlopen(req) as response:
                    response_data = response.read()
                    response_status = response.getcode()

                self.send_response(response_status)
                for key, value in response.getheaders():
                    self.send_header(key, value)
                self.end_headers()
                self.wfile.write(response_data)
            except Exception as err:
                self.send_response(500)
                self.send_header('Content-type', 'application/json')
                self.end_headers()
                error_response = '{{"errcode": 1, "errmsg": "Error relaying request: {}"}}'.format(str(err))
                self.wfile.write(error_response.encode('utf-8'))
        else:
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            success_response = '{"errcode": 0, "errmsg": "ok"}'
            self.wfile.write(success_response.encode('utf-8'))


def parse_address(addr: str) -> Tuple[str, int]:
    """Parse address string into host and port (backward compatibility)"""
    if addr.startswith(':'):
        host = ''
        port = int(addr[1:])
    else:
        parts = addr.split(':')
        host = parts[0]
        port = int(parts[1]) if len(parts) > 1 else 8000
    return host, port


def validate_port(port: int) -> None:
    """Validate port number"""
    if port < 1 or port > 65535:
        sys.stderr.write("Error: Port must be between 1 and 65535\n")
        sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(
        description='HTTP Relay Server - Forward POST requests to target URL',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
Examples:
  python main.py                                    # Listen on all interfaces, port 8000
  python main.py --port 8080                       # Listen on port 8080
  python main.py --host 127.0.0.1 --port 9000     # Listen on localhost:9000
  python main.py --url http://example.com          # Forward to example.com
  python main.py -v --url http://example.com       # Verbose mode with forwarding
  WEBOT_URL=http://example.com python main.py      # Use environment variable for URL
        '''
    )
    parser.add_argument('--host', default='', help='Host to bind to (default: all interfaces)')
    parser.add_argument('--port', type=int, default=8000, help='Port to bind to (default: 8000)')
    parser.add_argument('--url', help='URL to relay requests to')
    parser.add_argument('-v', '--verbose', action='store_true', help='Enable verbose logging')
    parser.add_argument('addr', nargs='?', help='[Deprecated] Address format (use --host/--port instead)')

    args = parser.parse_args()

    if args.addr:
        sys.stderr.write("Warning: 'addr' positional argument is deprecated. Use --host and --port instead.\n")
        host, port = parse_address(args.addr)
        if not args.host:
            args.host = host
        if args.port == 8000 or args.port == parse_address(args.addr)[1]:
            args.port = port

    validate_port(args.port)

    url = args.url
    if not url:
        url = os.environ.get('WEBOT_URL')

    RelayHTTPRequestHandler.url = url
    RelayHTTPRequestHandler.verbose = args.verbose

    server = HTTPServer((args.host, args.port), RelayHTTPRequestHandler)

    if args.verbose:
        sys.stderr.write("Server running on {}:{}\n".format(args.host or '0.0.0.0', args.port))

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        sys.stderr.write("\nServer stopped.\n")

if __name__ == "__main__":
    main()
