import * as fs from 'fs'


/**
 * @param {string} path
 * @param {string} target
 * */
const clone = (path, target) => {
  if (fs.existsSync(target)) {
    fs.rmSync(target, { recursive: true, force: true });
  }

  fs.cpSync(path, target, { recursive: true });
}

const editVersion = () => {
  const srcPath = './package.json';
  const targetPath = './build/package.json';

  if (fs.existsSync(targetPath)) {
    fs.rmSync(targetPath, { recursive: true, force: true });
  }

  const packageJson = fs.readFileSync(srcPath, 'utf8');

  const packageJsonParsed = JSON.parse(packageJson);

  packageJsonParsed.dependencies['@subwallet/invariant-vara-sdk-wasm'] = packageJsonParsed.version;

  fs.writeFileSync(targetPath, JSON.stringify(packageJsonParsed, null, 2), 'utf8');
}

clone('./target', './build/target');
clone('./contracts', './build/contracts');
clone('./README.md', './build/README.md');
editVersion();
