const { execSync } = require('child_process');
const fs = require('fs');
const https = require('https');

async function makeApiRequest(prompt, apiKey, model) {
  const postData = JSON.stringify({
    contents: [{
      parts: [{ text: prompt }]
    }]
  });

  const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${apiKey}`;

  return new Promise((resolve, reject) => {
    const req = https.request(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData)
      }
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => resolve({ statusCode: res.statusCode, body: data }));
    });

    req.on('error', reject);
    req.write(postData);
    req.end();
  });
}

async function askGeminiWithRetry(prompt, apiKey) {
  // Try gemini-3.5-flash first, fallback to gemini-3-flash-preview if needed
  const models = ['gemini-3.5-flash', 'gemini-3-flash-preview'];
  let delay = 2000;
  
  for (const model of models) {
    console.log(`Sending request using model: ${model}...`);
    for (let attempt = 1; attempt <= 2; attempt++) {
      try {
        const { statusCode, body } = await makeApiRequest(prompt, apiKey, model);
        const resJson = JSON.parse(body);
        
        if (statusCode === 200) {
          if (!resJson.candidates || resJson.candidates.length === 0) {
            throw new Error("No candidates returned from Gemini API");
          }
          return resJson.candidates[0].content.parts[0].text.trim();
        }
        
        const errMsg = resJson.error ? resJson.error.message : 'Unknown API error';
        console.warn(`[${model}] Attempt ${attempt} failed with status ${statusCode}: ${errMsg}`);
        
        if (statusCode === 503 || statusCode === 429) {
          console.log(`Service busy. Waiting ${delay}ms before retrying...`);
          await new Promise(r => setTimeout(r, delay));
          delay *= 2;
        } else {
          throw new Error(`Gemini API returned status ${statusCode}: ${errMsg}`);
        }
      } catch (err) {
        console.warn(`Error on model ${model}, attempt ${attempt}: ${err.message}`);
        if (model === models[models.length - 1] && attempt === 2) {
          throw err; // Out of options
        }
      }
    }
  }
}

async function main() {
  const apiKey = process.env.GEMINI_API_KEY;
  if (!apiKey) {
    console.error("Error: GEMINI_API_KEY environment variable is not set.");
    process.exit(1);
  }

  // Get commit range from GitHub event
  const before = process.env.COMMIT_BEFORE;
  const after = process.env.COMMIT_AFTER || 'HEAD';
  const repo = process.env.GITHUB_REPOSITORY || 'Spectra010s/portal';
  
  console.log(`Resolving commit range: before=${before}, after=${after}, repo=${repo}`);
  
  let shas = [];
  try {
    if (!before || /^0+$/.test(before)) {
      console.log("No valid 'before' commit ref found (first push). Using last commit SHA.");
      shas = [execSync('git log -n 1 --pretty=format:"%H"').toString().trim()];
    } else {
      console.log(`Fetching commit SHAs between ${before} and ${after}...`);
      const output = execSync(`git log ${before}..${after} --no-merges --pretty=format:"%H"`).toString().trim();
      shas = output.split('\n').filter(Boolean);
    }
  } catch (err) {
    console.warn("Failed to get commit range, using last commit SHA...", err.message);
    shas = [execSync('git log -n 1 --pretty=format:"%H"').toString().trim()];
  }

  if (shas.length === 0) {
    console.log("No new commits found in this push. Exiting.");
    return;
  }

  // Resolve PR numbers for commits using 'gh api'
  const commitsList = [];
  for (const sha of shas) {
    const subject = execSync(`git log -1 --pretty=format:"%s" ${sha}`).toString().trim();
    const body = execSync(`git log -1 --pretty=format:"%b" ${sha}`).toString().trim();
    
    let prNumber = "";
    try {
      prNumber = execSync(`gh api repos/${repo}/commits/${sha}/pulls --jq ".[0].number" 2>/dev/null`).toString().trim();
    } catch (e) {
      // ignore or fallback
    }
    
    commitsList.push({ sha, subject, body, prNumber });
  }

  // Format commits into readable prompt text
  let commitsText = "";
  for (const c of commitsList) {
    commitsText += `Commit: ${c.sha.substring(0, 7)}\n`;
    commitsText += `Subject: ${c.subject}\n`;
    if (c.prNumber) {
      commitsText += `Associated Pull Request: #${c.prNumber}\n`;
    }
    if (c.body) {
      commitsText += `Body:\n${c.body}\n`;
    }
    commitsText += `-----------\n`;
  }

  console.log("Formatted commits for Gemini prompt:\n", commitsText);

  // Construct prompt for Gemini
  const prompt = `You are a changelog assistant. You will be given a list of new git commits pushed to the repository.
Your task is to summarize these commits and format them exactly matching this Keep a Changelog line item style:
- **type(scope)**: Short description (#PR_NUMBER) [[short_hash](https://github.com/Spectra010s/portal/commit/commit_hash)]

Rules:
1. ONLY return the formatted markdown bullet points. Do not include any headers, explanation, or markdown fences (like \`\`\`markdown).
2. Differentiate between features (feat), fixes (fix), docs, build, chore, refactor, and build dependencies (build(deps)).
3. Group them if multiple commits are about the same thing, but keep their hashes and PR links.
4. If an "Associated Pull Request" is provided in the list (e.g. "Associated Pull Request: #123"), use that PR number as #PR_NUMBER. If no PR is associated, omit the "(#PR_NUMBER)" part.
5. Use the short commit hash in both the label and the URL link.

Here is the list of new commits:
${commitsText}`;

  const generatedText = await askGeminiWithRetry(prompt, apiKey);
  console.log("Generated changelog lines:\n", generatedText);

  // Update CHANGELOG.md
  console.log("Updating CHANGELOG.md...");
  const changelogPath = 'CHANGELOG.md';
  let changelog = fs.readFileSync(changelogPath, 'utf8');

  const unreleasedMarker = '## [Unreleased]';
  const index = changelog.indexOf(unreleasedMarker);
  if (index === -1) {
    console.error("Error: Could not find '## [Unreleased]' marker in CHANGELOG.md.");
    process.exit(1);
  }

  const insertPos = index + unreleasedMarker.length;
  let nextNewline = changelog.indexOf('\n', insertPos);
  if (nextNewline === -1) nextNewline = changelog.length;
  
  let insertAt = nextNewline + 1;
  if (changelog[insertAt] === '\n') {
    insertAt += 1;
  }

  const updatedChangelog = changelog.slice(0, insertAt) + generatedText + '\n' + changelog.slice(insertAt);
  fs.writeFileSync(changelogPath, updatedChangelog, 'utf8');
  console.log("CHANGELOG.md successfully updated!");
}

main().catch(err => {
  console.error("Unhandled error:", err);
  process.exit(1);
});
