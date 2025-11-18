from utils import load_data, transform_data

def main_entry():
    data = load_data()
    result = process_data(data)
    output_result(result)

def process_data(data):
    return transform_data(data)

def output_result(result):
    print(f"Result: {result}")

def unused_function():
    print("This function is never called")
