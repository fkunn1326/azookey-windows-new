import subprocess
import os

def check_wsb():
    result = subprocess.run(['wsb.exe', 'list'], capture_output=True, text=True)
    first_line = result.stdout.splitlines()[0]
    return first_line

def execute_wsb(sandbox_id, command):
    result = subprocess.run(['wsb.exe', 'exec', '--id', sandbox_id, '-c', command, '--run-as', 'ExistingLogin'], capture_output=True, text=True)
    return result.stdout

def main():
    # check if wsb is running
    print("checking wsb status...")
    sandbox_id = check_wsb()

    if not sandbox_id:
        print("wsb not running, starting...")
        # `wsb.exe start` isn't working properly...
        subprocess.run(['C:/Windows/system32/WindowsSandbox.exe'])
        while not check_wsb():
            pass
        sandbox_id = check_wsb()

        # share the folder
        bin_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../target/release')
        log_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../log')
        subprocess.run(['wsb.exe', 'share', '--id', sandbox_id, '-f', bin_path])
        subprocess.run(['wsb.exe', 'share', '--id', sandbox_id, '-f', log_path, '-w'])
    
    print(f"wsb is running id: {sandbox_id}")

    # execute the command
    execute_wsb(sandbox_id, r'xcopy C:\Users\WDAGUtilityAccount\Desktop\release C:\Users\WDAGUtilityAccount\Desktop\IME /e /y')
    execute_wsb(sandbox_id, r'regsvr32.exe C:\Users\WDAGUtilityAccount\Desktop\IME\ime.dll /s')


if __name__ == '__main__':
    main()