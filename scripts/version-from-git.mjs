import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

function git(args) {
  return execSync(`git ${args}`, { encoding: 'utf8' }).trim();
}

const date = git('log -1 --format=%cd --date=format:%Y.%-m.%-d');
const sha = git('rev-parse --short HEAD');
const version = `${date}+g${sha}`;

const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
if (pkg.version === version) {
  console.log(`version already ${version}`);
} else {
  const from = pkg.version;
  pkg.version = version;
  writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
  console.log(`version ${from} -> ${version}`);
}
