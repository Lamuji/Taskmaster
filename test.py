# simple_script.py
import time
import random

while True:
    print("Working...")
    time.sleep(2)
    if random.random() < 0.1:
        print("Error: Something went wrong!", flush=True)
    else:
        print("Success: Task completed successfully.", flush=True)

