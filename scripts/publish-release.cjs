const { execSync } = require('child_process');
const readline = require('readline');

// 获取版本号参数
const version = process.argv[2];

if (!version) {
  console.error('Usage: node scripts/publish-release.cjs <version>');
  console.error('Example: node scripts/publish-release.cjs 1.0.0');
  process.exit(1);
}

const tag = `v${version}`;

console.log(`🚀 Publishing release ${tag}...`);

try {
  // 1. 检查 draft release 是否存在
  console.log('\n📋 Checking draft release...');
  const releaseInfo = execSync(`gh release view ${tag} --json isDraft`, { encoding: 'utf-8' });
  const release = JSON.parse(releaseInfo);

  if (!release.isDraft) {
    console.error(`❌ Draft release ${tag} not found!`);
    process.exit(1);
  }

  // 2. 列出所有附件
  console.log('\n📦 Release assets:');
  const assetsInfo = execSync(`gh release view ${tag} --json assets`, { encoding: 'utf-8' });
  const assets = JSON.parse(assetsInfo).assets;

  assets.forEach(asset => {
    const sizeMB = Math.floor(asset.size / 1024 / 1024);
    console.log(`  - ${asset.name} (${sizeMB}MB)`);
  });

  // 3. 确认发布
  console.log('');
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });

  rl.question('❓ Publish this release? (y/N): ', (answer) => {
    rl.close();

    if (answer.toLowerCase() !== 'y') {
      console.log('❌ Release not published');
      process.exit(0);
    }

    // 4. 发布
    console.log('\n🚀 Publishing release...');
    execSync(`gh release edit ${tag} --draft=false`, { stdio: 'inherit' });

    const releaseUrl = execSync(`gh release view ${tag} --json url -q .url`, { encoding: 'utf-8' }).trim();

    console.log('');
    console.log(`✅ Release ${tag} published successfully!`);
    console.log(`🔗 ${releaseUrl}`);
  });

} catch (error) {
  console.error('\n❌ Publish failed:', error.message);
  process.exit(1);
}
