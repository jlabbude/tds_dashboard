const esbuild = require('esbuild');

esbuild.build({
  entryPoints: ['js/chart.js'],
  bundle: true,
  outfile: 'src/package.js',
  format: 'esm',
  minify: true,
}).catch(() => process.exit(1));