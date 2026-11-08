"""Server di prova che imita Open-Meteo su 127.0.0.1 (solo per scripts/test-oggi.ps1). Ogni richiesta finisce nel registro.
Uso: python weather-mock-server.py <porta> <registro>
/v1/search?name=Nowhere... -> nessun risultato; qualunque altro nome -> Roma. /v1/forecast -> previsioni fisse."""
import http.server
import json
import sys
import urllib.parse

port, log = int(sys.argv[1]), sys.argv[2]

GEO = {"results": [{"id": 1, "name": "Roma", "latitude": 41.89193, "longitude": 12.51133, "country": "Italia"}]}
FC = {
    "current": {"time": "2026-11-09T00:15", "interval": 900, "temperature_2m": 10.1, "weather_code": 3},
    "daily": {"time": ["2026-11-09"], "temperature_2m_max": [17.3], "temperature_2m_min": [9.8]},
}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        u = urllib.parse.urlparse(self.path)
        q = urllib.parse.parse_qs(u.query)
        with open(log, "a", encoding="utf-8") as f:
            f.write("GET %s %s\n" % (u.path, q.get("name", [""])[0]))
        if u.path == "/v1/search":
            body = {"generationtime_ms": 0.1} if q.get("name", [""])[0].startswith("Nowhere") else GEO
        elif u.path == "/v1/forecast":
            body = FC
        else:
            self.send_response(404)
            self.end_headers()
            return
        data = json.dumps(body).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *a):
        pass


http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
