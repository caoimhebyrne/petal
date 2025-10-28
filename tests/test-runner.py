#!/usr/bin/python3

#
# This is a script that checks that the test cases within this directory produce the expected bytecode.
#
# The script requires that the `tests` directory has a neighboring `target` directory. The `target` directory can either
# contain the petal compiler built in `release` or `debug` mode.
#
# If a release build of the compiler is avaialble, that will be used over a debug build.
#

from os import walk
from dataclasses import dataclass
from pathlib import Path
from typing import IO

import subprocess

tests_directory: Path = Path(__file__).parent

def log_info(message: str):
    print(f'\033[1;34mINFO:\033[0m {message}')

def log_pass(message: str):
    print(f'\033[1;32mPASS:\033[0m {message}')

def log_fail(message: str):
    print(f'\033[1;31mFAIL:\033[0m {message}')

def log_error(message: str):
    print(f'\033[1;31mERROR:\033[0m {message}')

@dataclass
class TestCase:
    '''Contains information about each test case'''
    name: str
    source_file_path: Path
    bytecode: str

    def __init__(self, name: str, source_file_path: Path, bytecode: str):
        self.name = name
        self.source_file_path = source_file_path
        self.bytecode = bytecode.strip()

    def execute(self, compiler_path: Path) -> bool:
        print()
        log_info(f'Running test {self.name}')

        # We know that the compiler exists, we can start a process pointing it to the source file.
        process = subprocess.Popen([compiler_path, '--dump-bytecode', self.source_file_path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        process.wait()

        stdout: str = ''
        if process.stdout:
            stdout = self.prepare_stdout(process.stdout)
        
        stderr: str = ''
        if process.stderr:
            stderr = process.stderr.read().strip()

        # If the process has exited with a non-zero exit code, then the test has failed.
        if not process.returncode == 0:
            log_fail(f'{self.name} has failed, process exited with code {process.returncode}, see output below.')

            if not len(stdout) == 0:
                print(f'\nStandard Output:\n{stdout}')

            if not len(stderr) == 0:
                print(f'\nStandard Error:\n{stderr}')

            return False

        # If the standard output does not equal the expected bytecode, then the test has failed.
        if not stdout == self.bytecode:
            log_fail(f'{self.name} has failed, compiler did not produce matching bytecode, see output below.')
            print(f'\nExpected:\n{self.bytecode}') 
            print(f'\nStandard Output:\n{stdout}')
            
            if not len(stderr) == 0:
                print(f'Standard Error:\n{stderr}')

            return False
        
        log_pass(f'{self.name} has passed!')
        return True

    def prepare_stdout(self, stdout: IO[str]) -> str:
        final_string: str = ''

        for line in stdout:
            if line.startswith('; ModuleID = ') or line.startswith('source_filename = '):
                continue

            final_string += line

        return final_string.strip()

        
# Each test case must end with the `.petal` extension, and the expected bytecode must end in `.b`.s
def collect_test_cases() -> list[TestCase]:
    # The 'cases' directory must exist beside this script.
    test_cases_directory: Path = tests_directory / 'cases'

    # All scripts use the `.petal` extension.
    petal_extension: str = '.petal'

    # ALl bytecode files use the `.petal.b` extension.
    bytecode_extension: str = f'.b'

    test_cases: list[TestCase] = []

    # We ignore the first two parameters, we are not interested in the root or any subdirectories.
    for _, _, files in walk(test_cases_directory):
        for file_name in files:
            if not file_name.endswith(petal_extension):
                continue

            # Each `.petal` file must have a corresponding `.petal.b` file.
            bytecode_file_name: str = f'{file_name}{bytecode_extension}'
            bytecode_file_path: Path = test_cases_directory / bytecode_file_name

            if not bytecode_file_path.exists():
                print(f'FATAL: {file_name} does not have a corresponding bytecode file ({bytecode_file_name})')
                exit(-1)

            bytecode: str = ''
            with open(bytecode_file_path, 'r') as bytecode_file:
                bytecode = bytecode_file.read()

            # We can then construct a test case 
            test_case_name = file_name.removesuffix(petal_extension)
            source_file_path: Path = test_cases_directory / file_name

            test_cases.append(TestCase(test_case_name, source_file_path, bytecode))
    
    return test_cases

# Attempts to find a petal compiler in a neighbouring directory to the tests directory.
def find_petal_compiler() -> Path:
    target_directory: Path = tests_directory.parent / 'target'

    # If a release executable exists, we will prioritise that over the debug executable.
    executable_path: Path

    release_build: Path = target_directory / 'release' / 'petal'
    debug_build: Path = target_directory / 'debug' / 'petal'

    if release_build.exists():
        executable_path = release_build
    elif debug_build.exists():
        executable_path = debug_build
    else:
        print(f'FATAL: Could not find a petal compiler in {release_build} or {debug_build}.')
        exit(-1)

    return executable_path

def main():
    compiler_path: Path = find_petal_compiler()
    log_info(f'Using petal compiler located at {compiler_path}')

    test_cases: list[TestCase] = collect_test_cases()
    log_info(f'Starting test suite with {len(test_cases)} test{'' if len(test_cases) == 1 else 's'}')

    success: bool = True

    for case in test_cases:
        # We store the failure state to be able to exit with an invalid status code later.
        if not case.execute(compiler_path):
            success = False

    if not success:
        print()
        log_error(f'One or more tests failed, see the above logs for more information.')
        exit(-1)

if __name__ == '__main__':
    main()
