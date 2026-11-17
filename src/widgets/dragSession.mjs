// Serialize IPC so a fast release cannot overtake begin_drag. Coalesce moves.
export function createDragSession(invoke) {
  let ending = false;
  let moving = false;
  let tail = Promise.resolve().then(() => invoke('begin_drag'));
  const ready = tail;
  return {
    ready,
    move() {
      if (ending || moving) return Promise.resolve();
      moving = true;
      tail = tail.then(() => invoke('update_drag'));
      return tail.finally(() => { moving = false; });
    },
    finish(cancel = false) {
      if (ending) return tail;
      ending = true;
      tail = tail.then(async () => {
        try { if (!cancel) await invoke('update_drag'); }
        finally { await invoke('finish_drag', { cancel }); }
      }, async error => {
        await invoke('finish_drag', { cancel: true });
        throw error;
      });
      return tail;
    },
  };
}
