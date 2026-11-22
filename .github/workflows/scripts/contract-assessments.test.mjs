// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { assessment, resolveGroup } from './contract-assessments.mjs';

const body = text => '<!-- contract-assessment -->\n```yaml\n' + text + '\n```';

// Scenario: an author edits or accidentally duplicates the assessment in a PR description.
// Guarantees: the current marked block is the only evidence and malformed markup cannot hide a partial assessment.
test('extracts the edited assessment', () => {
  assert.equal(assessment('ordinary PR'), '');
  assert.equal(assessment(body('original')), 'original');
  assert.equal(assessment(body('edited').replaceAll('\n', '\r\n')), 'edited');
  assert.throws(() => assessment(body('first') + '\n' + body('second')), /single/);
  assert.throws(() => assessment(body('edited').slice(0, -3)), /Unclosed/);
});

// Scenario: two independent PRs are squashed in order on top of the queue base.
// Guarantees: assessments refer to the individual heads, and a missing or changed queued head fails closed.
test('resolves the exact two-member squash group', t => {
  const previous = process.cwd();
  const dir = mkdtempSync(join(tmpdir(), 'contract-queue-'));
  t.after(() => { process.chdir(previous); rmSync(dir, { recursive: true, force: true }); });
  process.chdir(dir);
  const git = (...args) => execFileSync('git', args, { encoding: 'utf8', stdio: ['ignore','pipe','pipe'] }).trim();
  git('init', '-b', 'main');
  git('config', 'user.name', 'Test');
  git('config', 'user.email', 'test@example.invalid');
  git('config', 'commit.gpgsign', 'false');
  git('commit', '--allow-empty', '-m', 'base');
  const base = git('rev-parse', 'HEAD');
  const entries = [1, 2].map(number => {
    git('switch', '-c', 'member-' + number, base);
    writeFileSync(String(number), String(number));
    git('add', String(number));
    git('commit', '-m', 'feature ' + number);
    return { number, headRefOid: git('rev-parse', 'HEAD'), body: body('claim ' + number) };
  });
  git('switch', 'main');
  for (const e of entries) {
    git('merge', '--squash', e.headRefOid);
    git('commit', '-m', 'queue (#' + e.number + ')');
  }
  const group = { base_sha: base, head_sha: git('rev-parse','HEAD'), head_ref: 'refs/heads/gh-readonly-queue/main/pr-2-' + base };
  assert.deepEqual(resolveGroup(group, entries, 'test/project').map(m => [m.base, m.head, m.yaml]),
    entries.map(e => [base, e.headRefOid, 'claim ' + e.number]));
  assert.throws(() => resolveGroup(group, entries.slice(1), 'test/project'), /members/);
  assert.throws(() => resolveGroup(group, [entries[0], {...entries[1], headRefOid: entries[0].headRefOid}], 'test/project'), /does not reproduce/);
});
