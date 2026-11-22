// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Resolve PR metadata separately from the offline Rust checker. Treat PR text as data.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const git = (...args) => execFileSync('git', args, { encoding: 'utf8' }).trim();
const gh = (...args) => JSON.parse(execFileSync('gh', ['api', ...args], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
const oid = value => {
  if (!/^[a-f0-9]{40}$/.test(value)) throw Error('Expected a full GitHub commit SHA');
  return value;
};

export function assessment(body) {
  const lines = (body ?? '').replaceAll('\r\n', '\n').split('\n');
  const markers = lines.flatMap((line, i) => line.trim() === '<!-- contract-assessment -->' ? [i] : []);
  if (markers.length === 0) return '';
  if (markers.length !== 1) throw Error('Expected a single contract assessment');
  let start = markers[0] + 1;
  while (start < lines.length && !lines[start].trim()) start++;
  if (lines[start]?.trim() !== '```yaml') throw Error('Expected a YAML fence after the contract assessment marker');
  const end = lines.findIndex((line, i) => i > start && line.trim() === '```');
  if (end === -1) throw Error('Unclosed contract assessment');
  return lines.slice(start + 1, end).join('\n');
}

function fetchHead(repo, number, head) {
  oid(head);
  try {
    git('cat-file', '-e', head + '^{commit}');
  } catch {
    git('fetch', '--no-tags', 'origin', 'refs/pull/' + number + '/head');
    if (git('rev-parse', 'FETCH_HEAD') !== head) throw Error(repo + '#' + number + ' changed while collecting evidence');
  }
}

function member(repo, number, base, head, body) {
  fetchHead(repo, number, head);
  return { url: 'https://github.com/' + repo + '/pull/' + number, base: git('merge-base', base, head), head, yaml: assessment(body) };
}

// For this repository's squash queue there is one first-parent commit per PR.
// Verify every queued head reproduces its synthetic commit's tree on its actual parent.
export function resolveGroup(group, entries, repo) {
  const tail = Number(group.head_ref.match(/\/pr-(\d+)-[a-f0-9]+$/)?.[1]);
  const last = entries.findIndex(e => e.number === tail);
  let parent = oid(group.base_sha);
  const candidate = oid(group.head_sha);
  const chain = git('rev-list', '--first-parent', '--reverse', parent + '..' + candidate).split('\n');
  if (!tail || last < 0 || last + 1 < chain.length) throw Error('Cannot establish all merge-group members');
  const members = [];
  const used = new Set();
  for (const [i, sha] of chain.entries()) {
    const pr = entries[last + 1 - chain.length + i];
    if (used.has(pr.number) || git('rev-parse', oid(sha) + '^1') !== parent)
      throw Error('Ambiguous merge-group membership at ' + sha);
    used.add(pr.number);
    const subject = git('show', '-s', '--format=%s', sha);
    if (pr.queueHead !== sha && !subject.endsWith(' (#' + pr.number + ')')) {
      throw Error('Cannot associate queue commit ' + sha + ' with PR #' + pr.number);
    }
    const entry = member(repo, pr.number, group.base_sha, oid(pr.headRefOid), pr.body);
    const tree = git('merge-tree', '--write-tree', parent, entry.head);
    if (tree !== git('rev-parse', sha + '^{tree}')) throw Error('Queued PR #' + pr.number + ' does not reproduce ' + sha);
    members.push(entry);
    parent = sha;
  }
  return members;
}

function queue(repo, baseRef) {
  const [owner, name] = repo.split('/');
  const query = `
    query($owner: String!, $name: String!, $branch: String!, $endCursor: String) {
      repository(owner: $owner, name: $name) {
        mergeQueue(branch: $branch) {
          entries(first: 100, after: $endCursor) {
            nodes { position headCommit { oid } pullRequest { number headRefOid body } }
            pageInfo { hasNextPage endCursor }
          }
        }
      }
    }`;
  const pages = gh('graphql', '--paginate', '--slurp', '-f', 'query=' + query, '-f', 'owner=' + owner, '-f', 'name=' + name, '-f', 'branch=' + baseRef.replace(/^refs\/heads\//, ''));
  return pages.flatMap(page => page.data.repository.mergeQueue.entries.nodes)
    .sort((left, right) => left.position - right.position)
    .map(entry => ({ ...entry.pullRequest, queueHead: entry.headCommit?.oid }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
  const repo = process.env.GITHUB_REPOSITORY;
  const group = event.merge_group;
  if (git('rev-parse', 'HEAD') !== oid(process.env.GITHUB_SHA)) throw Error('Unexpected checkout revision');
  if (group && group.head_sha !== process.env.GITHUB_SHA) throw Error('Stale merge-group event');
  const pr = event.pull_request;
  let members;
  if (group) {
    members = resolveGroup(group, queue(repo, group.base_ref), repo);
  } else {
    // A PR event tests the merge revision. Bind its mutable description to the
    // exact base and contributor head used to create that revision.
    const parents = git('show', '-s', '--format=%P', process.env.GITHUB_SHA).split(' ');
    if (parents.length !== 2 || parents[0] !== oid(pr.base.sha) || parents[1] !== oid(pr.head.sha)) {
      throw Error('The PR assessment and tested merge revision have different parents');
    }
    members = [member(repo, pr.number, pr.base.sha, pr.head.sha, pr.body)];
  }
  process.stdout.write(JSON.stringify(members));
}
