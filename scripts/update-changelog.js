const { execSync } = require('child_process');
const fs = require('fs');
const https = require('https');

async function main() {
  const apiKey = process.env.GEMINI_API_KEY;
  if (!apiKey) {
    console.error("Error: GEMINI_API_KEY environment variable is not set.");
    process.exit(1);
  }

  // Get commit range from GitHub event
  const before = process.env.COMMIT_BEFORE;
  const after = process.env.COMMIT_AFTER || 'HEAD';
  
  console.log(`Resolving commit range: before=${before}, after=${after}`);
  
  let commits = "";
  try {
    // If before is all zeros (new branch/tag push) or empty, fall back to last commit
    if (!before || /^0+$/.test(before)) {
      console.log("No valid 'before' commit ref found (first push). Using last commit.");
      commits = execSync(`git log -n 1 --no-merges --pretty=format:"- %s (%h)%n%b%n---%n"`).toString().trim();
    } else {
      console.log(`Fetching commits between ${before} and ${after}...`);
      commits = execSync(`git log ${before}..${after} --no-merges --pretty=format:"- %s (%h)%n%b%n---%n"`).toString().trim();
    }
  } catch (err) {
    console.warn("Failed to get commit range, falling back to last commit...", err.message);
    commits = execSync(`git log -n 1 --no-merges --pretty=format:"- %s (%h)%n%b%n---%n"`).toString().trim();
  }

  if (!commits) {
    console.log("No new commits found in this push. Exiting.");
    return;
  }

  console.log("Commits to summarize:\n", commits);

  // Construct prompt for Gemini
  const prompt = `You are a changelog assistant. You will be given a list of new git commits pushed to the repository.
Your task is to summarize these commits and format them exactly matching this Keep a Changelog line item style:
- **type(scope)**: Short description (#PR_NUMBER) [[short_hash](https://github.com/Spectra010s/portal/commit/commit_hash)]

Rules:
1. ONLY return the formatted markdown bullet points. Do not include any headers, explanation, or markdown fences (like \`\`\`markdown).
2. Differentiate between features (feat), fixes (fix), docs, build, chore, refactor, and build dependencies (build(deps)).
3. Group them if multiple commits are about the same thing, but keep their hashes and PR links.
4. Extract the PR number from the commit message if present (e.g., if message has "(#123)", extract 123 as the PR number). If no PR number is present, omit the "(#PR_NUMBER)" part.
5. Use the short commit hash in both the label and the URL link.

Here is the list of new commits:
${commits}`;

  console.log("Sending request to Gemini API...");
  
  const postData = JSON.stringify({
    contents: [{
      parts: [{ text: prompt }]
    }]
  });

  const url = `https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash:generateContent?key=${apiKey}`;

  const responseText = await new Promise((resolve, reject) => {
    const req = https.request(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData)
      }
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => resolve(data));
    });

    req.on('error', reject);
    req.write(postData);
    req.end();
  });

  const resJson = JSON.parse(responseText);
  if (!resJson.candidates || resJson.candidates.length === 0) {
    console.error("Gemini API Error: No candidates returned.", JSON.stringify(resJson));
    process.exit(1);
  }

  const generatedText = resJson.candidates[0].content.parts[0].text.trim();
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

  // Insert the new lines right after '## [Unreleased]\n\n' or '## [Unreleased]\n'
  const insertPos = index + unreleasedMarker.length;
  
  // Find where the next line starts
  let nextNewline = changelog.indexOf('\n', insertPos);
  if (nextNewline === -1) nextNewline = changelog.length;
  
  // Check if there is an empty line after, if so skip it
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
