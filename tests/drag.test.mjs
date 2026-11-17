import test from 'node:test';
import assert from 'node:assert/strict';
import { createDragSession } from '../src/widgets/dragSession.mjs';
test('release waits for initialization, final movement and persistence', async () => {
  const calls = []; let start;
  const session = createDragSession(async (command, args) => {
    calls.push([command, args]);
    if (command === 'begin_drag') await new Promise(resolve => { start = resolve; });
  });
  const done = session.finish(); await Promise.resolve();
  assert.deepEqual(calls, [['begin_drag', undefined]]);
  start(); await done;
  assert.deepEqual(calls.map(c => c[0]), ['begin_drag', 'update_drag', 'finish_drag']);
  assert.deepEqual(calls.at(-1)[1], { cancel: false });
});
test('Escape cancels without another movement and only finishes once', async () => {
  const calls = [];
  const session = createDragSession(async (command, args) => calls.push([command, args]));
  await session.ready;
  await Promise.all([session.finish(true), session.finish(false)]);
  assert.deepEqual(calls, [['begin_drag', undefined], ['finish_drag', { cancel: true }]]);
});
test('movement failure still releases native drag session', async () => {
  const calls = [];
  const session = createDragSession(async (command, args) => {
    calls.push([command, args]);
    if (command === 'update_drag') throw new Error('window closed');
  });
  await session.ready;
  await assert.rejects(session.move(), /window closed/);
  await assert.rejects(session.finish(), /window closed/);
  assert.deepEqual(calls.at(-1), ['finish_drag', { cancel: true }]);
});
