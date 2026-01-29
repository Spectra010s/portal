from flask import Flask, jsonify, request, send_file, abort, Response
from flask_cors import CORS
import os
import tempfile
import zipfile
import shutil
import base64

app = Flask(__name__)
CORS(app)

# ====== CONFIG ======
BASE_DIR = os.path.abspath(os.path.expanduser("."))  # current directory (where index.html is)
PASSWORD = "563789" # password
# the username input isnt used but makes any intruder thinks he/she needs username to get in

def check_auth():
    auth = request.headers.get("Authorization")
    if not auth or not auth.startswith("Basic "):
        return False
    try:
        encoded = auth.split(" ")[1]
        decoded = base64.b64decode(encoded).decode("utf-8")
        user, pw = decoded.split(":", 1)
        return pw == PASSWORD
    except Exception:
        return False

def require_auth():
    return Response(
        "Authentication required", 401,
        {"WWW-Authenticate": 'Basic realm="Login Required"'}
    )

@app.before_request
def auth_middleware():
    if request.path.startswith("/api") or request.path.startswith("/download") or request.path.startswith("/download_zip") or request.path == "/":
        if not check_auth():
            return require_auth()

def safe_path(path):
    joined = os.path.abspath(os.path.join(BASE_DIR, path.lstrip("/")))
    if not joined.startswith(BASE_DIR):
        raise ValueError("Unsafe path")
    return joined

@app.route("/api/list", methods=["GET"])
def list_dir():
    relpath = request.args.get("path", "")
    try:
        path = safe_path(relpath)
    except ValueError:
        return jsonify({"error": "unsafe path"}), 400

    if not os.path.exists(path):
        return jsonify({"error": "not found"}), 404

    if os.path.isfile(path):
        return jsonify({"error": "not a directory"}), 400

    items = []
    for name in sorted(os.listdir(path)):
        full = os.path.join(path, name)
        items.append({
            "name": name,
            "is_dir": os.path.isdir(full),
            "size": os.path.getsize(full) if os.path.isfile(full) else None,
            "path": os.path.relpath(full, BASE_DIR).replace("\\", "/")
        })
    return jsonify({"path": os.path.relpath(path, BASE_DIR).replace("\\", "/"), "items": items})

@app.route("/download", methods=["GET"])
def download_file():
    relpath = request.args.get("path", "")
    try:
        path = safe_path(relpath)
    except ValueError:
        return abort(400)
    if not os.path.exists(path) or not os.path.isfile(path):
        return abort(404)
    # as_attachment=False allows the browser to display the media inline
    return send_file(path, as_attachment=True, conditional=True)

@app.route("/download_zip", methods=["GET"])
def download_zip():
    relpath = request.args.get("path", "")
    try:
        path = safe_path(relpath)
    except ValueError:
        return abort(400)
    if not os.path.exists(path):
        return abort(404)

    if os.path.isfile(path):
        return send_file(path, as_attachment=True, download_name=os.path.basename(path), conditional=True)

    tmpdir = tempfile.mkdtemp(prefix="ziptmp_")
    try:
        tmpzip = os.path.join(tmpdir, "archive.zip")
        with zipfile.ZipFile(tmpzip, "w", zipfile.ZIP_DEFLATED) as zf:
            for root, dirs, files in os.walk(path):
                for f in files:
                    full = os.path.join(root, f)
                    arcname = os.path.relpath(full, path)
                    zf.write(full, arcname)
        return send_file(tmpzip, as_attachment=True, download_name=f"{os.path.basename(path)}.zip")
    finally:
        try:
            shutil.rmtree(tmpdir)
        except Exception:
            pass

@app.route("/", methods=["GET"])
def index():
    index_path = os.path.join(BASE_DIR, "index.html")
    if not os.path.isfile(index_path):
        return "index.html not found", 404
    return send_file(index_path)

if __name__ == "__main__":
    HOST = "0.0.0.0"
    PORT = 8080
    print("Serving:", BASE_DIR)
    app.run(host=HOST, port=PORT, debug=False)
