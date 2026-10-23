// QA still renderer: node qa.mjs name=frame name=frame ...
//   node qa.mjs 02-hero-250=250 04-invoice-425=425
import { execSync } from 'node:child_process';

const specs = process.argv.slice(2);
for (const spec of specs) {
  const eq = spec.lastIndexOf('=');
  const name = spec.slice(0, eq);
  const frame = spec.slice(eq + 1);
  const cmd = `npx remotion still src/index.ts LedgerlyPromo out/qa/${name}.png --frame=${frame}`;
  try {
    execSync(cmd, { stdio: 'pipe' });
    console.log(`ok   ${name} (f${frame})`);
  } catch (e) {
    console.log(`FAIL ${name} (f${frame})`);
    const out = `${e.stdout ?? ''}${e.stderr ?? ''}`;
    console.log(out.split('\n').slice(-14).join('\n'));
  }
}
