const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");

const AdmZip = require("adm-zip");
const tar = require("tar");

function download(url, destPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destPath);
    const req = https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        file.close(() => {
          fs.rmSync(destPath, { force: true });
          resolve(download(res.headers.location, destPath));
        });
        return;
      }

      if (res.statusCode !== 200) {
        reject(new Error(`HTTP ${res.statusCode} downloading ${url}`));
        return;
      }

      res.pipe(file);
      file.on("finish", () => file.close(resolve));
    });

    req.on("error", (err) => {
      file.close(() => reject(err));
    });
  });
}

function ensureDir(p) {
  fs.mkdirSync(p, { recursive: true });
}

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "win32" && arch === "x64") {
    return { name: "win32-x64", archive: "kura-windows-x64.zip" };
  }
  if (platform === "linux" && arch === "x64") {
    return { name: "linux-x64", archive: "kura_voice-linux-x64.tar.gz" };
  }
  if (platform === "darwin") {
    if (arch === "arm64") {
      return { name: "macos-arm64", archive: "kura-macos-arm64.tar.gz" };
    }
    if (arch === "x64") {
      return { name: "macos-x64", archive: "kura-macos-x64.tar.gz" };
    }
  }

  return null;
}

async function install() {
  const info = getPlatformInfo();
  if (!info) {
    throw new Error(`Unsupported platform/arch: ${process.platform}-${process.arch}`);
  }

  const pkgJsonPath = path.join(__dirname, "..", "package.json");
  const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, "utf8"));
  const version = pkg.version;
  const tag = `v${version}`;

  const vendorDir = path.join(__dirname, "..", "vendor", info.name);
  const ext = process.platform === "win32" ? ".exe" : "";
  const kuraExe = path.join(vendorDir, `kura${ext}`);
  const kuracExe = path.join(vendorDir, `kurac${ext}`);

  if (fs.existsSync(kuraExe) && fs.existsSync(kuracExe)) {
    if (fs.statSync(kuraExe).size > 0 && fs.statSync(kuracExe).size > 0) {
      return { kuraExe, kuracExe };
    }
  }

  const localTargetDir = path.join(__dirname, "..", "..", "..", "target", "release");
  const localKura = path.join(localTargetDir, `kura${ext}`);
  const localKurac = path.join(localTargetDir, `kurac${ext}`);

  if (fs.existsSync(localKura) && fs.existsSync(localKurac)) {
    console.log(`@toto-sys28/kura: Using local binaries from ${localTargetDir}`);
    ensureDir(vendorDir);
    fs.copyFileSync(localKura, kuraExe);
    fs.copyFileSync(localKurac, kuracExe);
    if (process.platform !== "win32") {
      fs.chmodSync(kuraExe, 0o755);
      fs.chmodSync(kuracExe, 0o755);
    }
    return { kuraExe, kuracExe };
  }

  ensureDir(vendorDir);

  const url = `https://github.com/TOTO-sys28/KurA/releases/download/${tag}/${info.archive}`;
  const tmpArchive = path.join(vendorDir, info.archive);

  console.log(`@toto-sys28/kura: downloading ${url}`);
  await download(url, tmpArchive);

  if (info.archive.endsWith(".zip")) {
    const zip = new AdmZip(tmpArchive);
    zip.extractEntryTo(`kura${ext}`, vendorDir, false, true);
    zip.extractEntryTo(`kurac${ext}`, vendorDir, false, true);
  } else {
    await tar.x({
      file: tmpArchive,
      cwd: vendorDir,
      filter: (p) => p === "kura" || p === "kurac",
    });
    fs.chmodSync(kuraExe, 0o755);
    fs.chmodSync(kuracExe, 0o755);
  }

  fs.rmSync(tmpArchive, { force: true });

  return { kuraExe, kuracExe };
}

module.exports = { install };
