import logging
import os
import shlex
import subprocess
import typing
from pathlib import Path


class ProcessExecutor:
    def __init__(self, command):
        self.command = command

    @staticmethod
    def execute_command(command: str, cwd: Path, environment: typing.Dict[str, str], use_nix_shell: bool) -> str:
        environ = dict(os.environ)
        environ.update(environment)
        logging.info("Executing : " + command)
        if use_nix_shell:
            command = ["nix-shell", "--run", command]
            result = subprocess.run(
                command=command,
                cwd=cwd,
                encoding="utf-8",
                env=environ,
                capture_output=True,
                text=True,
            )
        else:
            command = shlex.split(command)
            result = subprocess.run(command, cwd=cwd, encoding="utf-8", capture_output=True, text=True, env=environ)

        if result.returncode > 1:
            logging.error("Process Executor failed for " + str(command))
            logging.error(result.stderr)
            logging.error(result.stdout)
            raise subprocess.CalledProcessError(result.returncode, command, result.args, result.stderr)
        else:
            return result.stdout.strip()