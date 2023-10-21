import os
import sys

script_dir = os.path.dirname(__file__)
sys.path.append(script_dir)

import registration

def main():
    print("Run registration flow test")
    registration.run()

if __name__ == '__main__':
    main()
