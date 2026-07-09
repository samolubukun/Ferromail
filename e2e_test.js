const { spawn, execSync } = require('child_process');
const path = require('path');

const API_URL = 'http://localhost:3000';
let apiProcess = null;
let workerProcess = null;

// Cleanup helper to terminate background processes on exit
function cleanup() {
  console.log('Cleaning up processes...');
  if (apiProcess) {
    try {
      apiProcess.kill();
      console.log('Killed API server process.');
    } catch (e) {}
  }
  if (workerProcess) {
    try {
      workerProcess.kill();
      console.log('Killed Worker process.');
    } catch (e) {}
  }
}

process.on('exit', cleanup);
process.on('SIGINT', () => {
  cleanup();
  process.exit(1);
});
process.on('uncaughtException', (err) => {
  console.error('Uncaught Exception:', err);
  cleanup();
  process.exit(1);
});

async function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Function to poll the API health check until it is online
async function waitForServer(url, timeoutMs = 30000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(`${url}/health`);
      if (res.ok) {
        const body = await res.json();
        if (body.status === 'ok') {
          return true;
        }
      }
    } catch (e) {
      // Server not ready yet
    }
    await sleep(1000);
  }
  throw new Error(`Timeout waiting for server at ${url}`);
}

async function runTests() {
  console.log('=== Starting Ferromail End-to-End Test Suite ===');

  const backendCwd = path.join(__dirname, 'backend');

  // Ensure all binaries are fully built first to avoid concurrent cargo lock contention
  console.log('Ensuring all binaries are fully built sequentially...');
  execSync('cargo build --workspace', { cwd: backendCwd, stdio: 'inherit' });

  // 1. Launch API Server
  console.log('Launching API Server...');
  apiProcess = spawn(path.join(backendCwd, 'target', 'debug', 'api.exe'), [], {
    cwd: backendCwd,
    env: { ...process.env },
    stdio: 'inherit'
  });

  // 2. Launch Worker Daemon
  console.log('Launching Background Worker...');
  workerProcess = spawn(path.join(backendCwd, 'target', 'debug', 'worker.exe'), [], {
    cwd: backendCwd,
    env: { ...process.env },
    stdio: 'inherit'
  });

  // 3. Wait for boot
  console.log('Waiting for API Server to become healthy...');
  await waitForServer(API_URL, 60000);
  console.log('API Server is healthy and online!');

  // Generate random email to avoid duplicate key conflicts
  const randomSuffix = Math.floor(Math.random() * 1000000);
  const email = `e2e_owner_${randomSuffix}@ferromail.com`;
  const password = 'securepassword123';

  // Test 1: User Signup
  console.log(`\nTest 1: Signing up user ${email}...`);
  const signupRes = await fetch(`${API_URL}/v1/auth/signup`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });

  if (!signupRes.ok) {
    throw new Error(`Signup failed with status ${signupRes.status}: ${await signupRes.text()}`);
  }

  const signupData = await signupRes.json();
  if (!signupData.success || !signupData.token || !signupData.project_id) {
    throw new Error(`Signup payload invalid: ${JSON.stringify(signupData)}`);
  }
  const token = signupData.token;
  const projectId = signupData.project_id;
  console.log(`Success: Registered user. Project ID: ${projectId}`);

  // Test 2: List Contacts (should be empty initially)
  console.log('\nTest 2: Listing contacts (should be empty)...');
  const listRes = await fetch(`${API_URL}/v1/contacts`, {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  });

  if (!listRes.ok) {
    throw new Error(`Listing contacts failed: ${await listRes.text()}`);
  }
  const contacts = await listRes.json();
  console.log(`Success: Listed ${contacts.length} contacts.`);

  // Test 3: Create Contact
  const contactEmail = `subscriber_e2e_${randomSuffix}@ferromail.com`;
  console.log(`\nTest 3: Creating contact ${contactEmail}...`);
  const createContactRes = await fetch(`${API_URL}/v1/contacts`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      email: contactEmail,
      data: { firstName: 'Alice' },
    }),
  });

  if (!createContactRes.ok) {
    throw new Error(`Creating contact failed: ${await createContactRes.text()}`);
  }
  const contactData = await createContactRes.json();
  console.log(`Success: Contact created with ID ${contactData.id}`);

  // Test 4: Verify Contact in List
  console.log('\nTest 4: Re-listing contacts to verify creation...');
  const verifyListRes = await fetch(`${API_URL}/v1/contacts`, {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });
  const verifiedContacts = await verifyListRes.json();
  const found = verifiedContacts.find(c => c.email === contactEmail);
  if (!found || found.data.firstName !== 'Alice') {
    throw new Error(`Verification failed. Contact list state: ${JSON.stringify(verifiedContacts)}`);
  }
  console.log('Success: Contact verified in contact list.');

  // Test 5: Send Transactional Email
  console.log('\nTest 5: Triggering transactional email send...');
  const sendRes = await fetch(`${API_URL}/v1/send`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      from: `sender_${randomSuffix}@ferromail.com`,
      to: contactEmail,
      subject: `Hello from E2E Test Suite ${randomSuffix}`,
      body: 'Hi {{firstName}}, this is a template variable test!',
    }),
  });

  if (!sendRes.ok) {
    throw new Error(`Send email failed: ${await sendRes.text()}`);
  }
  const sendData = await sendRes.json();
  if (!sendData.success || !sendData.email_ids || sendData.email_ids.length === 0) {
    throw new Error(`Invalid send response: ${JSON.stringify(sendData)}`);
  }
  const emailId = sendData.email_ids[0];
  console.log(`Success: Email queued in database. ID: ${emailId}`);

  // Test 6: Verify background worker delivery processing
  console.log('\nTest 6: Waiting for background worker to process delivery...');
  await sleep(4000); // Wait for worker loop cycle

  console.log('Querying database state of the email directly...');
  const dbQueryCommand = `docker exec ferromail-db psql -U postgres -d postgres -t -A -c "SELECT status, body FROM emails WHERE id = '${emailId}';"`;
  const dbResult = execSync(dbQueryCommand).toString().trim();
  console.log(`Database response: ${dbResult}`);

  const [status, body] = dbResult.split('|');
  if (status !== 'DELIVERED') {
    throw new Error(`Email status in database is not DELIVERED. Got: ${status}`);
  }
  if (!body.includes('Hi Alice, this is a template variable test!')) {
    throw new Error(`Email template fields were not correctly rendered. Rendered body: ${body}`);
  }
  console.log('Success: Email marked as DELIVERED in database.');
  console.log('Success: Email template variables correctly compiled ("Hi Alice").');

  // Test 7: Verify event logs
  console.log('\nTest 7: Verifying event logs are recorded...');
  const eventQueryCommand = `docker exec ferromail-db psql -U postgres -d postgres -t -A -c "SELECT name FROM events WHERE \\"emailId\\" = '${emailId}';"`;
  const eventResult = execSync(eventQueryCommand).toString().trim();
  console.log(`Database event: ${eventResult}`);
  if (eventResult !== 'email.delivered') {
    throw new Error(`Event log not found or incorrect. Got: ${eventResult}`);
  }
  console.log('Success: Verification event log `email.delivered` is present.');

  console.log('\n=== All Integration Tests Completed Successfully! ===');
}

runTests()
  .then(() => {
    cleanup();
    process.exit(0);
  })
  .catch((err) => {
    console.error('\n❌ E2E Integration Test Failed:');
    console.error(err);
    cleanup();
    process.exit(1);
  });
