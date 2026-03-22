from flask import Flask, request, render_template
import os

app = Flask(__name__)

UPLOAD_FOLDER = os.path.expanduser("~/namtes")
os.makedirs(UPLOAD_FOLDER, exist_ok=True)

@app.route("/", methods=["GET", "POST"])
def upload_files():
    if request.method == "POST":
        if "files" not in request.files:
            return render_template("pup.html", message="No files part")

        uploaded_files = request.files.getlist("files")
        saved_files = []

        for file in uploaded_files:
            if file.filename:
                save_path = os.path.join(UPLOAD_FOLDER, file.filename)
                file.save(save_path)
                saved_files.append(file.filename)

        if saved_files:
            msg = f"Uploaded successfully, thank you"
        else:
            msg = "No files selected."
        return render_template("pup.html", message=msg)

    return render_template("pup.html", message=None)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)