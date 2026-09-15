"""Test the Windows archive format on every host with real dated files."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from release_archive import create_archive


class ReleaseArchiveTests(unittest.TestCase):
    def test_zip_preserves_old_license_contents_and_clamps_stored_dates(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            package = root / "mom-test-live-x86_64-pc-windows-msvc"
            notices = package / "licenses"
            notices.mkdir(parents=True)
            notice = notices / "LICENSE"
            notice.write_bytes(b"Required license notice\n")
            os.utime(notice, (86400, 86400))
            (package / "mom-test-live.exe").write_bytes(b"test executable contents")
            archive = create_archive(package, root / "release", "zip")
            with zipfile.ZipFile(archive) as bundle:
                self.assertIsNone(bundle.testzip())
                member = package.name + "/licenses/LICENSE"
                self.assertEqual(bundle.read(member), notice.read_bytes())
                self.assertEqual(bundle.getinfo(member).date_time[:3], (1980, 1, 1))
                self.assertEqual(bundle.read(package.name + "/mom-test-live.exe"),
                                 b"test executable contents")
            self.assertEqual(notice.stat().st_mtime, 86400)


if __name__ == "__main__":
    unittest.main()
