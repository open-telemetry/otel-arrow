"""Run the selected engine binary locally or reuse the image built by Rust CI."""

import argparse
from contextlib import contextmanager
import os
from pathlib import Path
import subprocess
import uuid


class Engine:
    def __init__(self, binary, image):
        self.binary = binary
        self.image = image

    def command(self, config, *, live=False, name=None):
        if self.binary:
            return [self.binary, "--config", config]
        # Only the test directory is visible inside the container. Keep host paths
        # identical so the file exporter's generation-specific captures are observable.
        directory = config.resolve().parent
        command = ["docker", "run", "--rm", "--read-only", "--network", "host" if live else "none",
                   "--user", f"{os.getuid()}:{os.getgid()}",
                   "--volume", f"{directory}:{directory}:{'rw' if live else 'ro'}"]
        if name:
            command.extend(["--name", name])
        return command + [self.image, "--config", config.resolve()]

    @contextmanager
    def running(self, config, log):
        name = f"operator-guarantees-{uuid.uuid4().hex}"
        process = subprocess.Popen(self.command(config, live=True, name=name),
                                   stdout=log, stderr=subprocess.STDOUT)
        try:
            yield process
        finally:
            if self.image and process.poll() is None:
                subprocess.run(["docker", "rm", "--force", name], check=True, timeout=10,
                               stdout=subprocess.DEVNULL)
            process.kill()
            process.wait()


def from_args(description):
    parser = argparse.ArgumentParser(description=description)
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument("binary", type=Path, nargs="?")
    choice.add_argument("--image", help="existing Linux engine image on this Docker host")
    args = parser.parse_args()
    return Engine(args.binary.resolve() if args.binary else None, args.image)
