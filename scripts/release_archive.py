"""Archive native releases, including crate notices with pre-1980 timestamps."""
from pathlib import Path
import shutil
import zipfile


def create_archive(package: Path, destination: Path, archive_format: str) -> Path:
    if archive_format != "zip":
        return Path(shutil.make_archive(str(destination), archive_format,
                                        package.parent, package.name))
    archive = destination.with_name(destination.name + ".zip")
    # Cargo crate archives often date license files to 1970. ZIP's date field
    # starts at 1980; clamp stored dates without changing the source files.
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED,
                         strict_timestamps=False) as output:
        for path in [package, *sorted(package.rglob("*"))]:
            output.write(path, path.relative_to(package.parent))
    return archive
