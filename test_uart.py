import serial
import time
import sys

port = '/dev/tty.usbmodem84302'
try:
    s = serial.Serial(port, 115200, timeout=1)
    s.dtr = True
    s.rts = True
    print(f"Connected to {port}. Waiting for data...")
    t0 = time.time()
    while time.time() - t0 < 5:
        line = s.readline()
        if line:
            print(f"RX: {line}")
            t0 = time.time()
        else:
            time.sleep(0.1)
    print("No more data (timeout).")
except Exception as e:
    print(f"Error: {e}")
