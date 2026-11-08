"""Server di prova per gli aggiornamenti: serve una cartella su 127.0.0.1 e scrive ogni richiesta in un file di registro.
Uso: python update-feed-server.py <cartella> <porta> <registro>
Solo per le prove (test-update.ps1): ascolta soltanto su localhost."""
import http.server
import sys

root, port, log = sys.argv[1], int(sys.argv[2]), sys.argv[3]


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *a, **k):
        super().__init__(*a, directory=root, **k)

    def log_message(self, fmt, *args):
        with open(log, "a", encoding="utf-8") as f:
            f.write("%s %s\n" % (self.command, self.path))


http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
