import os
import sys
import argparse
from http.server import BaseHTTPRequestHandler, HTTPServer
import urllib.request
from typing import Tuple


class RelayHTTPRequestHandler(BaseHTTPRequestHandler):
    url = None
    verbose = False

    def do_POST(self) -> None:
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length)
        request_headers = dict(self.headers)

        # request logging
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

                # response logging
                if self.__class__.verbose:
                    sys.stderr.write("Response Status: {}\n".format(response_status))
                    sys.stderr.write("Response Headers:\n")
                    for key, value in response.getheaders():
                        sys.stderr.write("{}: {}\n".format(key, value))
                    sys.stderr.write("\nResponse Body:\n")
                    try:
                        sys.stderr.write(response_data.decode('utf-8'))
                    except UnicodeDecodeError:
                        sys.stderr.write("Binary data: {} bytes\n".format(len(response_data)))
                        sys.stderr.write(str(response_data))

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
            self.send_response(404)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            error_response = '{"errcode": 1, "errmsg": "URL not configured"}'
            self.wfile.write(error_response.encode('utf-8'))


def parse_bind_address(bind_addr: str) -> Tuple[str, int]:
    """Parse bind address string into host and port"""
    if not bind_addr:
        return '', 8000

    if bind_addr.startswith(':'):
        return '', int(bind_addr[1:])

    parts = bind_addr.rsplit(':', 1)
    if len(parts) == 2:
        host = parts[0]
        try:
            port = int(parts[1])
            return host, port
        except ValueError:
            return bind_addr, 8000

    return bind_addr, 8000


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
  python main.py --bind :8080                      # Listen on port 8080
  python main.py --bind 127.0.0.1:9000            # Listen on localhost:9000
  python main.py --bind :8000 --url http://example.com
  python main.py --bind :8000 -v                   # Verbose mode
  WEBOT_URL=http://example.com python main.py --bind :8000
        '''
    )
    parser.add_argument('-b', '--bind', default=':8000', help='Address to bind to (e.g., :8000 or localhost:8000)')
    parser.add_argument('--url', help='URL to relay requests to')
    parser.add_argument('-v', '--verbose', action='store_true', help='Enable verbose logging')

    args = parser.parse_args()

    host, port = parse_bind_address(args.bind)
    validate_port(port)

    url = args.url
    if not url:
        url = os.environ.get('WEBOT_URL')

    RelayHTTPRequestHandler.url = url
    RelayHTTPRequestHandler.verbose = args.verbose

    server = HTTPServer((host, port), RelayHTTPRequestHandler)

    if args.verbose:
        sys.stderr.write("Server running on {}:{}\n".format(host or '0.0.0.0', port))

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        sys.stderr.write("\nServer stopped.\n")

if __name__ == "__main__":
    main()
