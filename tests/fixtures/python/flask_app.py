from flask import Flask

app = Flask(__name__)

def handle_get_users():
    return [{"id": 1, "name": "John"}]

def validate_user(user):
    if not user.get("name"):
        raise ValueError("Name required")

def error_handler(error):
    return {"error": str(error)}, 500

@app.route('/users', methods=['GET'])
def get_users():
    return handle_get_users()

@app.route('/users', methods=['POST'])
def create_user():
    validate_user({"name": "Jane"})
    return {"id": 2}, 201

app.register_error_handler(500, error_handler)
