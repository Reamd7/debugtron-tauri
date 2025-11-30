const fs = require('fs');
const path = require('path');

try {
  // 读取 package.json 版本号
  const packageJsonPath = path.join(__dirname, '../package.json');
  if (!fs.existsSync(packageJsonPath)) {
    throw new Error('package.json not found!');
  }

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  const version = packageJson.version;

  if (!version) {
    throw new Error('Version not found in package.json!');
  }

  // 更新 Cargo.toml 版本号
  const cargoTomlPath = path.join(__dirname, '../src-tauri/Cargo.toml');
  if (!fs.existsSync(cargoTomlPath)) {
    throw new Error('Cargo.toml not found!');
  }

  let cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');
  cargoToml = cargoToml.replace(
    /^version = ".*"/m,
    `version = "${version}"`
  );
  fs.writeFileSync(cargoTomlPath, cargoToml);

  // 更新 tauri.conf.json 版本号
  const tauriConfPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
  if (!fs.existsSync(tauriConfPath)) {
    throw new Error('tauri.conf.json not found!');
  }

  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  tauriConf.package.version = version;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2));

  console.log(`✅ Synced version to ${version}`);
} catch (error) {
  console.error('❌ Failed to sync version:', error.message);
  process.exit(1);
}
