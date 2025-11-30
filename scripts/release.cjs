const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// 获取版本号参数
const version = process.argv[2];

if (!version) {
  console.error('Usage: node scripts/release.cjs <version>');
  console.error('Example: node scripts/release.cjs 1.0.0');
  process.exit(1);
}

// 验证版本号格式
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error('❌ Invalid version format! Please use semantic versioning (e.g., 1.0.0)');
  process.exit(1);
}

// 读取当前版本号
const packageJsonPath = path.join(__dirname, '../package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const currentVersion = packageJson.version;

if (currentVersion === version) {
  console.error(`❌ Version ${version} is already the current version!`);
  console.error('Please specify a different version number.');
  process.exit(1);
}

console.log(`📦 Preparing release v${version}...`);
console.log(`   Current version: ${currentVersion}`);
console.log(`   New version: ${version}`);

try {
  // 1. 更新版本号
  console.log('\n📝 Updating version numbers...');
  execSync(`npm version ${version} --no-git-tag-version`, { stdio: 'inherit' });

  // 2. 提交
  console.log('\n💾 Committing version changes...');
  execSync('git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json', { stdio: 'inherit' });
  execSync(`git commit -m "chore: release v${version}"`, { stdio: 'inherit' });

  // 3. 创建 tag
  console.log(`\n🏷️  Creating tag v${version}...`);
  execSync(`git tag v${version}`, { stdio: 'inherit' });

  // 4. 推送
  console.log('\n⬆️  Pushing to GitHub...');
  execSync('git push origin master', { stdio: 'inherit' });
  execSync(`git push origin v${version}`, { stdio: 'inherit' });

  console.log('\n');
  console.log('✅ Tag v' + version + ' created and pushed!');
  console.log('');
  console.log('📋 Next steps:');
  console.log(`  1. On macOS machine: ./scripts/build-and-upload.sh ${version}`);
  console.log(`  2. On Windows machine: .\\scripts\\build-and-upload.ps1 ${version}`);
  console.log(`  3. After both platforms complete: node scripts/publish-release.cjs ${version}`);
  console.log('');

} catch (error) {
  console.error('\n❌ Release failed:', error.message);
  process.exit(1);
}
