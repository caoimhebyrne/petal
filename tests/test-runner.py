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
    print(f'\033[1;32mPASS: {message}\033[0m')

def log_fail(message: str):
    print(f'\033[1;31mFAIL: {message}\033[0m')

def log_error(message: str):
    print(f'\033[1;31mERROR:\033[0m {message}')

@dataclass
class TestCaseOptions:
    '''Contains options provided in the comment header of each test case'''
    expected_exit_code: int
    compile_failure: str

    def __init__(self):
        self.expected_exit_code = 0
        self.compile_failure = ''

    @staticmethod
    def parse(source_file_path: Path) -> TestCaseOptions:
        options: TestCaseOptions = TestCaseOptions()
        source_file_contents: str = source_file_path.read_text()

        for line in source_file_contents.splitlines():
            # If the line does not start with a `// #` we can assume that this is the end of the test assertions.
            if not line.startswith('// #'):
                break
            
            options.parse_option(line)
        
        return options
                

    def parse_option(self, line: str):
        option_declaration: list[str] = line.removeprefix('// #').strip().split(': ')

        # There must be an option name on the left, and maybe a value on the right.
        option_name = option_declaration[0]

        option_value: str = ''
        if len(option_declaration) >= 2:
            option_value = option_declaration[1]

        if not option_name:
            log_error(f'{line} is not a valid test option declaration!')
            exit(-1)

        match option_name:
            case 'exit-code':
                self.expected_exit_code = int(option_value)

            case 'compile-failure':
                self.compile_failure = option_value

            case _:
                log_info(f'Unrecognized test option declaration: \'{option_declaration}\'')

@dataclass
class TestCase:
    '''Contains information about each test case'''
    name: str
    source_file_path: Path
    output_executable_path: Path
    options: TestCaseOptions

    def __init__(self, name: str, source_file_path: Path, output_executable_path: Path):
        self.name = name
        self.source_file_path = source_file_path
        self.output_executable_path = output_executable_path
        self.options = TestCaseOptions.parse(self.source_file_path)

    def execute(self, compiler_path: Path) -> bool:
        print()
        log_info(f'Running test {self.source_file_path.name}')

        # We know that the compiler exists, we can start a process pointing it to the source file.
        compile_process = subprocess.Popen([compiler_path, '--dump-bytecode', '-o', self.output_executable_path, self.source_file_path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        compile_process.wait()

        compiler_stdout: str = ''
        if compile_process.stdout:
            compiler_stdout = compile_process.stdout.read().strip()
        
        compiler_stderr: str = ''
        if compile_process.stderr:
            compiler_stderr = compile_process.stderr.read().strip()

        if not len(self.options.compile_failure) == 0:
            if compile_process.returncode == 0:
                log_fail(f'{self.name} has failed. A compilation failure was expected, but the compilation completed successfully.')
                return False
            
            if self.options.compile_failure in compiler_stderr:
                log_pass(f'{self.name} has passed!')
                return True
            
            if not len(self.options.compile_failure) == 0:
                log_fail(f'{self.name} has failed. The expected compilation failure was not observed.')
                print(f'\nTest expected the following compilation error, but it was not present: \n{self.options.compile_failure}')

            if not len(compiler_stdout) == 0:
                print(f'\nStandard Output:\n{compiler_stdout}')

            if not len(compiler_stderr) == 0:
                print(f'\nStandard Error:\n{compiler_stderr}')

            return False

        # If the compiler has exited with a non-zero exit code, then the test has failed.
        if not compile_process.returncode == 0:
            log_fail(f'{self.name} has failed, compiler exited with code {compile_process.returncode}, see output below.')

            if not len(compiler_stdout) == 0:
                print(f'\nStandard Output:\n{compiler_stdout}')

            if not len(compiler_stderr) == 0:
                print(f'\nStandard Error:\n{compiler_stderr}')

            return False
        
        # We should now have a valid binary at the executable path.
        process = subprocess.Popen([self.output_executable_path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        process.wait()

        process_stdout: str = ''
        if process.stdout:
            process_stdout = process.stdout.read().strip()
        
        process_stderr: str = ''
        if process.stderr:
            process_stderr = process.stderr.read().strip()

        # If the compiler has exited with a non-zero exit code, then the test has failed.
        if not process.returncode == self.options.expected_exit_code:
            log_fail(f'{self.name} has failed, process exited with code {process.returncode} (expected {self.options.expected_exit_code}).')

            if not len(process_stdout) == 0:
                print(f'\nProcess Standard Output:\n{process_stdout}')

            if not len(process_stderr) == 0:
                print(f'\nProcess Standard Error:\n{process_stderr}')

            if not len(compiler_stdout) == 0:
                print(f'\nCompiler Standard Output:\n{compiler_stdout}')

            if not len(compiler_stderr) == 0:
                print(f'\nCompiler Standard Error:\n{compiler_stderr}')

            return False
        
        log_pass(f'{self.name} has passed!')
        return True

# Each test case must end with the `.petal` extension, and the expected bytecode must end in `.b`.s
def collect_test_cases(test_cases_directory: Path, executables_directory: Path) -> list[TestCase]:
    # All scripts use the `.petal` extension.
    petal_extension: str = '.petal'

    test_cases: list[TestCase] = []

    # We ignore the first two parameters, we are not interested in the root or any subdirectories.
    for _, _, files in walk(test_cases_directory):
        for file_name in files:
            if not file_name.endswith(petal_extension):
                continue

            # We can then construct a test case 
            test_case_name = file_name.removesuffix(petal_extension)
            source_file_path: Path = test_cases_directory / file_name
            output_executable_path = executables_directory / test_case_name

            test_cases.append(TestCase(test_case_name, source_file_path, output_executable_path))
    
    test_cases.sort(key=lambda it: it.name)
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

    # The 'cases' directory must exist beside this script.
    test_cases_directory: Path = tests_directory / 'cases'

    executables_directory: Path = tests_directory / '.petal-build'
    executables_directory.mkdir(exist_ok=True)

    test_cases: list[TestCase] = collect_test_cases(test_cases_directory, executables_directory)
    log_info(f'Starting test suite with {len(test_cases)} test{'' if len(test_cases) == 1 else 's'}')

    success: bool = True

    for case in test_cases:
        # We store the failure state to be able to exit with an invalid status code later.
        if not case.execute(compiler_path):
            success = False

    if success:
        for root, _, files in executables_directory.walk(top_down=False):
            for file in files:
                (root / file).unlink()

        executables_directory.rmdir()
    else:
        print()
        log_error(f'One or more tests failed, see the above logs for more information.')
        exit(-1)

if __name__ == '__main__':
    main()
