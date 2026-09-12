// Local Store's start script for LobeHub. It does three things LobeHub's own
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

// 2. The bucket LobeHub keeps uploads in. Upstream makes it with MinIO's `mc`
// image, which MinIO no longer publishes, so it is made here with a signed S3
// request instead — the same one `mc mb` sends.
const sha256 = (value) => crypto.createHash('sha256').update(value).digest('hex');
const hmac = (key, value) => crypto.createHmac('sha256', key).update(value).digest();

async function put(endpoint, path, body) {
  const url = new URL(path, endpoint);
  const now = new Date().toISOString().replace(/[:-]|\.\d{3}/g, '');
  const day = now.slice(0, 8);
  const region = process.env.S3_REGION || 'us-east-1';
  const scope = `${day}/${region}/s3/aws4_request`;
  const payload = sha256(body ?? '');
  const headers = {
    host: url.host,
    'x-amz-content-sha256': payload,
    'x-amz-date': now,
  };
  const signed = Object.keys(headers).sort();
  const canonical = [
    'PUT',
    url.pathname,
    url.search.slice(1),
    signed.map((name) => `${name}:${headers[name]}\n`).join(''),
    signed.join(';'),
    payload,
  ].join('\n');
  const toSign = ['AWS4-HMAC-SHA256', now, scope, sha256(canonical)].join('\n');
  const secret = process.env.S3_SECRET_ACCESS_KEY;
  let key = hmac(`AWS4${secret}`, day);
  for (const part of [region, 's3', 'aws4_request']) key = hmac(key, part);
  const signature = hmac(key, toSign).toString('hex');
  headers.authorization =
    `AWS4-HMAC-SHA256 Credential=${process.env.S3_ACCESS_KEY_ID}/${scope}, ` +
    `SignedHeaders=${signed.join(';')}, Signature=${signature}`;
  const response = await fetch(url, {
    method: 'PUT',
    headers,
    body,
    signal: AbortSignal.timeout(30000),
  });
  return { status: response.status, text: await response.text() };
}

async function ensureBucket() {
  const endpoint = process.env.S3_INTERNAL_ENDPOINT;
  const bucket = process.env.S3_BUCKET || 'lobe';
  const made = await put(endpoint, `/${bucket}`);
  // 409 is the bucket already being there, which every start after the first
  // one sees.
  if (made.status >= 400 && made.status !== 409) {
    throw new Error(`could not create the ${bucket} bucket: ${made.status} ${made.text.slice(0, 200)}`);
  }
  // Anything in the bucket is readable without a signature, which is what
  // upstream's `mc anonymous set download` does: LobeHub links to uploads
  // directly from the page.
  const policy = JSON.stringify({
    Version: '2012-10-17',
    Statement: [
      {
        Effect: 'Allow',
        Principal: { AWS: ['*'] },
        Action: ['s3:GetObject'],
        Resource: [`arn:aws:s3:::${bucket}/*`],
      },
    ],
  });
  const applied = await put(endpoint, `/${bucket}?policy=`, policy);
  if (applied.status >= 400) {
    console.error(`local-store: bucket policy refused (${applied.status}); uploads still work`);
  }
}

// 3. The object store at the address the browser uses. LobeHub has one S3
// address for both: the browser uploads to it, and the server reads files
// back from it. Inside this container that address would be LobeHub itself,
// so it is forwarded to the store.
const internal = new URL(process.env.S3_INTERNAL_ENDPOINT);
const port = Number(process.env.S3_LOCAL_PORT);
if (port) {
  net
    .createServer((client) => {
      const store = net.connect(Number(internal.port || 9000), internal.hostname);
      client.pipe(store).pipe(client);
      client.on('error', () => store.destroy());
      store.on('error', () => client.destroy());
    })
    .listen(port, '127.0.0.1');
}

// 4. LobeHub, started as its image's own command would.
ensureBucket().then(
  () => require('/app/startServer.js'),
  (error) => {
    console.error(`local-store: ${error.message}`);
    process.exit(1);
  },
);
