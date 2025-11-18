def load_data():
    return fetch_from_database()

def transform_data(data):
    cleaned = clean_data(data)
    return validate_data(cleaned)

def fetch_from_database():
    return "raw data from db"

def clean_data(data):
    return data.strip()

def validate_data(data):
    return f"validated: {data}"

def another_unused():
    print("Also never called")
