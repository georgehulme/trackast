class Calculator:
    def __init__(self):
        self.value = 0
    
    def add(self, x):
        self.validate()

    def validate(self):
        pass

class Logger:
    def __init__(self):
        self.logs = []
