// Local Store's start script for LobeHub. It does two things LobeHub's own
// Compose setup leaves to a script run by hand, then starts LobeHub exactly
// the way its image would.
'use strict';
const crypto = require('node:crypto');
const fs = require('node:fs');
const net = require('node:net');

// 1. A signing key of its own. LobeHub signs the calls its background jobs
// make back to it with this; without one, file processing fails. Made once,
// kept with the app's data, and reused on every start and reinstall.
const keyFile = '/keys/jwks.json';
if (!fs.existsSync(keyFile)) {
  const { privateKey } = crypto.generateKeyPairSync('rsa', { modulusLength: 2048 });
  const jwk = {
    ...privateKey.export({ format: 'jwk' }),
    alg: 'RS256',
    kid: crypto.randomBytes(8).toString('hex'),
    use: 'sig',
  };
  fs.writeFileSync(keyFile, JSON.stringify({ keys: [jwk] }), { mode: 0o600 });
}
process.env.JWKS_KEY = fs.readFileSync(keyFile, 'utf8');

// 2. The object store at the address the browser uses. LobeHub has one S3
// address for both: the browser uploads to it, and the server reads files
// back from it. Inside this container that address would be LobeHub itself,
// so it is forwarded to the store.
const port = Number(process.env.S3_LOCAL_PORT);
if (port) {
  net
    .createServer((client) => {
      const store = net.connect(9000, 'lobehub-store');
      client.pipe(store).pipe(client);
      client.on('error', () => store.destroy());
      store.on('error', () => client.destroy());
    })
    .listen(port, '127.0.0.1');
}

// 3. LobeHub, started as its image's own command would.
require('/app/startServer.js');
